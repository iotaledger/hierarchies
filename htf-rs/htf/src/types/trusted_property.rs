use iota_sdk::types::id::UID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrustedPropertyName {
  pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct TrustedPropertyValue {
  pub text: Option<String>,
  pub number: Option<u64>,
}
