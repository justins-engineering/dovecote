use serde::{Deserialize, Serialize};

static SERVICE_PLAN: &str = "free";

#[derive(Serialize, Deserialize)]
pub struct Flock {
  id: Option<i64>,
  name: String,
  service_plan: String,
  updated_at: Option<i64>,
  created_at: Option<i64>,
}

impl Default for Flock {
  fn default() -> Flock {
    Flock {
      id: Option::default(),
      name: String::with_capacity(64),
      service_plan: SERVICE_PLAN.to_string(),
      updated_at: Option::default(),
      created_at: Option::default(),
    }
  }
}
