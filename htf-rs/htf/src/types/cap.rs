use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::id::UID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthorityCap {
  id: UID,
  federation_id: ObjectID,
}
