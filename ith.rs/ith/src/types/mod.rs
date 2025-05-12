//! Types for the ITH protocol.

mod cap;
mod event;
mod permission;
mod trusted_statement;
mod trusted_statements;

pub use accreditation::*;
pub use cap::*;
pub use event::*;
pub use trusted_statement::*;
pub use trusted_statements::*;

use std::collections::HashMap;

use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::id::UID;
use serde::{Deserialize, Serialize};

use crate::utils::deserialize_vec_map;

/// Represents a federation. A federation is a group of entities that have agreed to work together
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Federation {
  pub id: UID,
  pub governance: Governance,
  pub root_authorities: Vec<RootAuthority>,
}

/// Represents a root authority. A root authority is an entity that has the highest level of authority in a federation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthority {
  pub id: UID,
  pub account_id: ObjectID,
}

/// Represents the governance of a federation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Governance {
  id: UID,
  pub trusted_statements: Statements,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub accreditors: HashMap<ObjectID, Accreditations>,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub attesters: HashMap<ObjectID, Accreditations>,
}
