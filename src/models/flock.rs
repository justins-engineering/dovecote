use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

static SERVICE_PLAN: &str = "free";

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Flock {
  pub id: String,
  pub user_id: String,
  pub name: String,
  pub service_plan: Option<String>,
  pub pigeon_count: i64,
  #[serde(with = "time::serde::rfc3339::option")]
  pub updated_at: Option<OffsetDateTime>,
  #[serde(with = "time::serde::rfc3339::option")]
  pub created_at: Option<OffsetDateTime>,
}

impl Default for Flock {
  fn default() -> Flock {
    Flock {
      id: String::with_capacity(64),
      user_id: String::with_capacity(64),
      name: String::with_capacity(64),
      service_plan: Some(SERVICE_PLAN.to_string()),
      pigeon_count: i64::default(),
      updated_at: Option::default(),
      created_at: Option::default(),
    }
  }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct CreateFlockPayload {
  pub name: String,
}
