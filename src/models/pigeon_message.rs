use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct PigeonMessage {
  pub id: i64,
  pub pigeon_id: i64,
  pub message: String,
  pub timestamp: i64,
}
