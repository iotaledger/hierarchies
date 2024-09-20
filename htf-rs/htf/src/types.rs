mod cap;
pub mod event;
pub mod permission;
pub mod trusted_constraints;
pub mod trusted_property;

use std::collections::HashMap;

use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::id::UID;
use permission::{PermissionsToAccredit, PermissionsToAttest};
use serde::{Deserialize, Serialize};
use trusted_constraints::TrustedPropertyConstraints;

use crate::utils::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Federation {
  pub id: UID,
  pub governance: Governance,
  pub root_authorities: Vec<RootAuthority>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthority {
  pub id: UID,
  pub account_id: ObjectID,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Governance {
  id: UID,
  pub trusted_constraints: TrustedPropertyConstraints,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub(crate) accreditors: HashMap<ObjectID, PermissionsToAccredit>,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub(crate) attesters: HashMap<ObjectID, PermissionsToAttest>,
}
