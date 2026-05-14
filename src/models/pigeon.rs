use serde::{Deserialize, Serialize};

static CONNECTOR: &str = "HTTPS";

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Pigeon {
  pub id: i64,
  pub flock_id: i64,
  pub name: String,
  pub serial: Option<String>,
  pub tags: Option<String>,
  pub connector: Option<String>,
  pub location: Option<String>,
  pub last_connected: Option<i64>,
  pub updated_at: Option<i64>,
  pub created_at: Option<i64>,
}

impl Default for Pigeon {
  fn default() -> Pigeon {
    Pigeon {
      id: i64::default(),
      flock_id: i64::default(),
      name: String::with_capacity(64),
      serial: Option::default(),
      tags: Option::default(),
      connector: Some(CONNECTOR.to_string()),
      location: Option::default(),
      last_connected: Option::default(),
      updated_at: Option::default(),
      created_at: Option::default(),
    }
  }
}
