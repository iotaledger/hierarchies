use iota_sdk::types::id::{ID, UID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthorityCap {
  id: UID,
  federation_id: ID,
}
