use serde::{Deserialize, Serialize};

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
