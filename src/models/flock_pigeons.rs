use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct FlockPigeon {
  pub id: i64,
  pub flock_id: i64,
  pub pigeon_id: i64,
  pub joined_at: i64,
}
