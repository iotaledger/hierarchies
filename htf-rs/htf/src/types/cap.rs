use iota_sdk::types::id::ID;
use iota_sdk::types::id::UID;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthorityCap {
  id: UID,
  federation_id: ID,
}
