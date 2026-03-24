use serde::{Deserialize, Serialize};
use serde_json;
use worker::{
  DurableObject, Env, Request, Response, Result, SqlStorage, State, console_error, durable_object,
  wasm_bindgen,
};

static CONNECTOR: &str = "HTTPS";

#[derive(Serialize, Deserialize)]
pub struct Pigeon {
  id: Option<i64>,
  name: String,
  serial: Option<String>,
  tags: Option<String>,
  connector: String,
  location: Option<String>,
  last_connected: Option<i64>,
  updated_at: Option<i64>,
  created_at: Option<i64>,
}

impl Default for Pigeon {
  fn default() -> Pigeon {
    Pigeon {
      id: Option::default(),
      name: String::with_capacity(64),
      serial: Option::default(),
      tags: Option::default(),
      connector: CONNECTOR.to_string(),
      location: Option::default(),
      last_connected: Option::default(),
      updated_at: Option::default(),
      created_at: Option::default(),
    }
  }
}

#[durable_object]
pub struct Pigeons {
  sql: SqlStorage,
  #[allow(unused)]
  state: State,
  #[allow(unused)]
  env: Env,
}

impl DurableObject for Pigeons {
  fn new(state: State, env: Env) -> Pigeons {
    let sql = state.storage().sql();
    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS pigeons (
          id TEXT NOT NULL PRIMARY KEY,
          serial TEXT,
          name TEXT,
          tags TEXT,
          connector TEXT,
          location TEXT,
          last_connected INTEGER,
          updated_at INTEGER DEFAULT (unixepoch()),
          created_at INTEGER DEFAULT (unixepoch())
        );

        CREATE TRIGGER prevent_immutable_updates
        BEFORE UPDATE OF id, created_at ON pigeons
        WHEN OLD.id IS NOT NEW.id
          OR OLD.created_at IS NOT NEW.created_at
        BEGIN
          SELECT RAISE(ABORT, 'Error: id and created_at columns are immutable.');
        END;

        CREATE TRIGGER set_updated_at
        AFTER UPDATE ON pigeons
        FOR EACH ROW
        WHEN NEW.updated_at = OLD.updated_at
        BEGIN
          UPDATE pigeons
          SET updated_at = unixepoch()
          WHERE id = OLD.id;
        END;

        CREATE INDEX IF NOT EXISTS idx_pigeons_serial ON pigeons(serial);
        CREATE INDEX IF NOT EXISTS idx_pigeons_name ON pigeons(name);
        CREATE INDEX IF NOT EXISTS idx_pigeons_tags ON pigeons(tags);
        CREATE INDEX IF NOT EXISTS idx_pigeons_last_connected ON pigeons(last_connected DESC);",
        None,
      )
      .expect("created pigeons table");

    sql
      .exec(
        "CREATE TABLE IF NOT EXISTS messages (
          id TEXT NOT NULL PRIMARY KEY,
          pigeon_id INTEGER NOT NULL,
          message TEXT NOT NULL,
          timestamp INTEGER DEFAULT (unixepoch()),
          FOREIGN KEY (pigeon_id) REFERENCES pigeons(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_messages_pigeon_id ON messages(pigeon_id);
        CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp DESC);
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(message);",
        None,
      )
      .expect("created messages table");

    Pigeons { sql, state, env }
  }

  async fn fetch(&self, req: Request) -> Result<Response> {
    match req.path().as_str() {
      "/pigeons/create" => create(self, req).await,
      "/pigeons/read_all" => read_all(self, req).await,
      "/pigeons/read" => read(self, req).await,
      "/pigeons/update" => update(self, req).await,
      "/pigeons/delete" => delete(self, req).await,
      "/pigeon/:id/messages" => read_messages(self, req).await,
      "/pigeon/:id/message/:id" => read_messages(self, req).await,
      _ => Response::error("Not found", 404),
    }
  }
}

async fn read_all(pigeons: &Pigeons, _req: Request) -> Result<Response> {
  let query = pigeons
    .sql
    .exec("SELECT * FROM pigeons;", None)?
    .to_array::<Vec<Pigeon>>();

  match query {
    Ok(rows) => match serde_json::to_string(&rows) {
      Ok(json) => Response::from_json(&json),
      Err(e) => {
        console_error!("Pigeon serialize error: {e}");
        Response::error("Internal Server Error", 500)
      }
    },
    Err(e) => {
      console_error!("Pigeons read error: {e}");
      Response::error("Internal Server Error", 500)
    }
  }
}

async fn read(pigeons: &Pigeons, mut req: Request) -> Result<Response> {
  match req.json::<Pigeon>().await {
    Ok(row) => {
      let query = pigeons
        .sql
        .exec("SELECT * FROM pigeons WHERE id = ?;", vec![row.id.into()])?
        .one::<Pigeon>();

      match query {
        Ok(pigeon) => match serde_json::to_string(&pigeon) {
          Ok(json) => Response::from_json(&json),
          Err(e) => {
            console_error!("Pigeon serialize error: {e}");
            Response::error("Internal Server Error", 500)
          }
        },
        Err(e) => {
          console_error!(
            "Pigeons read error: {e}\nRequest body: {:?}",
            req.text().await?
          );
          Response::error("Internal Server Error", 500)
        }
      }
    }
    Err(e) => {
      console_error!("Pigeons read error: {e}");
      Response::error("Bad Request", 400)
    }
  }
}

async fn create(pigeons: &Pigeons, mut req: Request) -> Result<Response> {
  match req.json::<Pigeon>().await {
    Ok(row) => {
      pigeons.sql.exec(
        "INSERT INTO pigeons (
            serial,
            name,
            tags,
            connector,
            location,
            last_connected
          )
          VALUES (?);",
        vec![
          row.serial.into(),
          row.name.into(),
          row.tags.into(),
          row.connector.into(),
          row.location.into(),
          row.last_connected.into(),
        ],
      )?;

      Response::empty()
    }
    Err(e) => {
      console_error!("Pigeons read error: {e}");
      Response::error("Bad Request", 400)
    }
  }
}

async fn update(pigeons: &Pigeons, mut req: Request) -> Result<Response> {
  match req.json::<Pigeon>().await {
    Ok(row) => {
      pigeons.sql.exec(
        "UPDATE pigeons SET
          serial=?,
          name=?,
          tags=?,
          connector=?,
          location=?,
          last_connected=?
          WHERE id = ?;",
        vec![
          row.serial.into(),
          row.name.into(),
          row.tags.into(),
          row.connector.into(),
          row.location.into(),
          row.last_connected.into(),
          row.id.into(),
        ],
      )?;

      Response::empty()
    }
    Err(e) => {
      console_error!("Pigeons read error: {e}");
      Response::error("Bad Request", 400)
    }
  }
}

async fn delete(pigeons: &Pigeons, mut req: Request) -> Result<Response> {
  match req.json::<Pigeon>().await {
    Ok(row) => {
      pigeons
        .sql
        .exec("DELETE FROM pigeons WHERE id = ?;", vec![row.id.into()])?;

      Response::empty()
    }
    Err(e) => {
      console_error!("Pigeons read error: {e}");
      Response::error("Bad Request", 400)
    }
  }
}

async fn read_messages(pigeons: &Pigeons, _req: Request) -> Result<Response> {
  let query = pigeons
    .sql
    .exec("SELECT * FROM pigeons;", None)?
    .to_array::<Vec<Pigeon>>();

  match query {
    Ok(rows) => match serde_json::to_string(&rows) {
      Ok(json) => Response::from_json(&json),
      Err(e) => {
        console_error!("Pigeon serialize error: {e}");
        Response::error("Internal Server Error", 500)
      }
    },
    Err(e) => {
      console_error!("Pigeons read error: {e}");
      Response::error("Internal Server Error", 500)
    }
  }
}
