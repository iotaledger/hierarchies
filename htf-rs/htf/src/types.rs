mod cap;
pub mod credentials;
pub mod event;
pub mod permission;
pub mod trusted_constraints;
pub mod trusted_property;

use std::collections::HashMap;

use crate::de::deserialize_vec_map;
use crate::utils::Hashable;

use credentials::CredentialState;

use iota_sdk::types::id::ID;
use iota_sdk::types::id::UID;
use permission::Permissions;
use permission::PermissionsToAccredit;
use permission::PermissionsToAttest;
use serde::Deserialize;
use serde::Serialize;
use trusted_constraints::TrustedPropertyConstraints;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Federation {
  pub id: UID,
  pub governance: Governance,
  pub root_authorities: Vec<RootAuthority>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthority {
  pub id: UID,
  pub account_id: ID,
  pub permissions: Permissions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Governance {
  id: UID,
  pub trusted_constraints: TrustedPropertyConstraints,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub(crate) accreditors: HashMap<Hashable<ID>, PermissionsToAccredit>,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub(crate) attesters: HashMap<Hashable<ID>, PermissionsToAttest>,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub(crate) credentials_state: HashMap<Hashable<ID>, CredentialState>,
}
