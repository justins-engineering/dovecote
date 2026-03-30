use serde::{Deserialize, Serialize};
use worker::{
  DurableObject, Env, Request, Response, ResponseBuilder, Result, SqlStorage, State, console_error,
  console_warn, durable_object, wasm_bindgen,
};

static SERVICE_PLAN: &str = "free";

#[derive(Serialize, Deserialize, Debug)]
pub struct Flock {
  id: i64,
  name: String,
  service_plan: Option<String>,
  updated_at: Option<i64>,
  created_at: Option<i64>,
}

impl Default for Flock {
  fn default() -> Flock {
    Flock {
      id: i64::default(),
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
          id INTEGER NOT NULL PRIMARY KEY,
          name TEXT NOT NULL,
          service_plan TEXT NOT NULL,
          updated_at INTEGER DEFAULT (unixepoch()),
          created_at INTEGER DEFAULT (unixepoch())
        );

        CREATE TRIGGER IF NOT EXISTS prevent_immutable_updates_on_flocks
        BEFORE UPDATE OF id, created_at ON flocks
        WHEN OLD.id IS NOT NEW.id
          OR OLD.created_at IS NOT NEW.created_at
        BEGIN
          SELECT RAISE(ABORT, 'Error: id and created_at columns are immutable.');
        END;

        CREATE TRIGGER IF NOT EXISTS set_updated_at
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
        "CREATE TABLE IF NOT EXISTS flock (
          id INTEGER NOT NULL PRIMARY KEY,
          flock_id INTEGER NOT NULL,
          pigeon_id TEXT NOT NULL,
          joined_at INTEGER DEFAULT (unixepoch()),
          FOREIGN KEY (flock_id) REFERENCES flocks(id) ON DELETE CASCADE
        );

        CREATE TRIGGER IF NOT EXISTS prevent_immutable_updates_on_flock
        BEFORE UPDATE OF id, pigeon_id ON flock
        WHEN OLD.id IS NOT NEW.id
          OR OLD.pigeon_id IS NOT NEW.pigeon_id
        BEGIN
          SELECT RAISE(ABORT, 'Error: id and pigeon_id columns are immutable.');
        END;

        CREATE INDEX IF NOT EXISTS idx_flock_flock_id ON flock(flock_id);
        CREATE INDEX IF NOT EXISTS idx_flock_pigeon_id ON flock(pigeon_id);
        CREATE INDEX IF NOT EXISTS idx_flock_joined_at ON flock(joined_at DESC);",
        None,
      )
      .expect("created flock table");

    Flocks { sql, state, env }
  }

  async fn fetch(&self, req: Request) -> Result<Response> {
    // console_warn!("{}", req.path().as_str());
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
    .to_array::<Flock>();

  match query {
    Ok(rows) => Response::from_json(&rows),
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
        Ok(flock) => Response::from_json(&flock),
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
      console_warn!("Row: {row:?}");
      let query = flocks
        .sql
        .exec(
          "INSERT INTO flocks (name, service_plan) VALUES (?, ?) RETURNING *;",
          vec![row.name.into(), row.service_plan.into()],
        )?
        .one::<Flock>();

      match query {
        Ok(flock) => {
          let mut location = String::with_capacity(72);
          location.push_str("/flocks/");
          location.push_str(&flock.id.to_string());

          ResponseBuilder::new()
            .with_status(201)
            .with_header("Location", &location)?
            .from_json(&flock)
        }
        Err(e) => {
          console_error!(
            "Flocks create error: {e}\nRequest body: {:?}",
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
