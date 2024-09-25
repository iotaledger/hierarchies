use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::id::UID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthorityCap {
  id: UID,
  federation_id: ObjectID,
}

/// Capabilities are the different types of capabilities that can be issued
/// to an account
#[derive(Debug, strum::Display, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capabilities {
  RootAuthority(RootAuthorityCap),
  #[strum(serialize = "AttestCap")]
  Attest,
  #[strum(serialize = "AccreditCap")]
  Accredit,
}
