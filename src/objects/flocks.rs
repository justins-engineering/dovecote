use serde::{Deserialize, Serialize};
use serde_json;
use worker::{
  DurableObject, Env, Request, Response, Result, SqlStorage, State, console_error, durable_object,
  wasm_bindgen,
};

static SERVICE_PLAN: &str = "free";

#[derive(Serialize, Deserialize)]
pub struct Flock {
  id: String,
  name: String,
  service_plan: Option<String>,
  updated_at: Option<i64>,
  created_at: Option<i64>,
}

impl Default for Flock {
  fn default() -> Flock {
    Flock {
      id: String::with_capacity(64),
      name: String::with_capacity(64),
      service_plan: Some(SERVICE_PLAN.to_string()),
      updated_at: Option::default(),
      created_at: Option::default(),
    }
  }
}

#[durable_object]
pub struct Flocks {
  sql: SqlStorage,
  #[allow(unused)]
  state: State,
  #[allow(unused)]
  env: Env,
}

impl DurableObject for Flocks {
  fn new(state: State, env: Env) -> Flocks {
    let sql = state.storage().sql();
    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS flocks (
          id TEXT NOT NULL PRIMARY KEY,
          name TEXT NOT NULL,
          service_plan TEXT NOT NULL,
          updated_at INTEGER DEFAULT (unixepoch()),
          created_at INTEGER DEFAULT (unixepoch())
        );

        CREATE TRIGGER prevent_immutable_updates
        BEFORE UPDATE OF id, created_at ON flocks
        WHEN OLD.id IS NOT NEW.id
          OR OLD.created_at IS NOT NEW.created_at
        BEGIN
          SELECT RAISE(ABORT, 'Error: id and created_at columns are immutable.');
        END;

        CREATE TRIGGER set_updated_at
        AFTER UPDATE ON flocks
        FOR EACH ROW
        WHEN NEW.updated_at = OLD.updated_at
        BEGIN
          UPDATE flocks
          SET updated_at = unixepoch()
          WHERE id = OLD.id;
        END;",
        None,
      )
      .expect("created flocks table");

    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS flock_pigeons (
          id TEXT NOT NULL PRIMARY KEY,
          flock_id INTEGER NOT NULL,
          pigeon_id TEXT NOT NULL,
          joined_at INTEGER DEFAULT (unixepoch()),
          FOREIGN KEY (flock_id) REFERENCES flocks(id) ON DELETE CASCADE
        );

        CREATE TRIGGER prevent_immutable_updates
        BEFORE UPDATE OF id, pigeon_id ON flock_pigeons
        WHEN OLD.id IS NOT NEW.id
          OR OLD.pigeon_id IS NOT NEW.pigeon_id
        BEGIN
          SELECT RAISE(ABORT, 'Error: id and pigeon_id columns are immutable.');
        END;

        CREATE INDEX IF NOT EXISTS idx_flock_pigeons_flock_id ON flock_pigeons(flock_id);
        CREATE INDEX IF NOT EXISTS idx_flock_pigeons_pigeon_id ON flock_pigeons(pigeon_id);
        CREATE INDEX IF NOT EXISTS idx_flock_pigeons_joined_at ON flock_pigeons(joined_at DESC);",
        None,
      )
      .expect("created flock_pigeons table");

    Flocks { sql, state, env }
  }

  async fn fetch(&self, req: Request) -> Result<Response> {
    match req.path().as_str() {
      "/flocks/create" => create(self, req).await,
      "/flocks/read_all" => read_all(self, req).await,
      "/flocks/read" => read(self, req).await,
      "/flocks/update" => update(self, req).await,
      "/flocks/delete" => delete(self, req).await,
      _ => Response::error("Not found", 404),
    }
  }
}

async fn read_all(flocks: &Flocks, _req: Request) -> Result<Response> {
  let query = flocks
    .sql
    .exec("SELECT * FROM flocks;", None)?
    .to_array::<Vec<Flock>>();

  match query {
    Ok(rows) => match serde_json::to_string(&rows) {
      Ok(json) => Response::from_json(&json),
      Err(e) => {
        console_error!("Flock serialize error: {e}");
        Response::error("Internal Server Error", 500)
      }
    },
    Err(e) => {
      console_error!("Flocks read error: {e}");
      Response::error("Internal Server Error", 500)
    }
  }
}

async fn read(flocks: &Flocks, mut req: Request) -> Result<Response> {
  match req.json::<Flock>().await {
    Ok(row) => {
      let query = flocks
        .sql
        .exec("SELECT * FROM flocks WHERE id = ?;", vec![row.id.into()])?
        .one::<Flock>();

      match query {
        Ok(flock) => match serde_json::to_string(&flock) {
          Ok(json) => Response::from_json(&json),
          Err(e) => {
            console_error!("Flock serialize error: {e}");
            Response::error("Internal Server Error", 500)
          }
        },
        Err(e) => {
          console_error!(
            "Flocks read error: {e}\nRequest body: {:?}",
            req.text().await?
          );
          Response::error("Internal Server Error", 500)
        }
      }
    }
    Err(e) => {
      console_error!("Flocks read error: {e}");
      Response::error("Bad Request", 400)
    }
  }
}

async fn create(flocks: &Flocks, mut req: Request) -> Result<Response> {
  match req.json::<Flock>().await {
    Ok(row) => {
      flocks.sql.exec(
        "INSERT INTO flocks (name, service_plan) VALUES (?);",
        vec![row.name.into(), row.service_plan.into()],
      )?;

      Response::empty()
    }
    Err(e) => {
      console_error!("Flocks read error: {e}");
      Response::error("Bad Request", 400)
    }
  }
}

async fn update(flocks: &Flocks, mut req: Request) -> Result<Response> {
  match req.json::<Flock>().await {
    Ok(row) => {
      flocks.sql.exec(
        "UPDATE flocks SET name=?, service_plan=? WHERE id = ?;",
        vec![row.name.into(), row.service_plan.into(), row.id.into()],
      )?;

      Response::empty()
    }
    Err(e) => {
      console_error!("Flocks read error: {e}");
      Response::error("Bad Request", 400)
    }
  }
}

async fn delete(flocks: &Flocks, mut req: Request) -> Result<Response> {
  match req.json::<Flock>().await {
    Ok(row) => {
      flocks
        .sql
        .exec("DELETE FROM flocks WHERE id = ?;", vec![row.id.into()])?;

      Response::empty()
    }
    Err(e) => {
      console_error!("Flocks read error: {e}");
      Response::error("Bad Request", 400)
    }
  }
}
