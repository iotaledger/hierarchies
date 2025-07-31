// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types for the Hierarchies protocol.

mod accreditation;
mod cap;
mod event;
pub mod statements;
pub mod timespan;

use std::collections::HashMap;

pub use accreditation::*;
pub use cap::*;
pub use event::*;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::id::UID;
use serde::{Deserialize, Serialize};

use crate::core::types::statements::Statements;
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
    pub id: UID,
    pub statements: Statements,
    #[serde(deserialize_with = "deserialize_vec_map")]
    pub accreditations_to_accredit: HashMap<ObjectID, Accreditations>,
    #[serde(deserialize_with = "deserialize_vec_map")]
    pub accreditations_to_attest: HashMap<ObjectID, Accreditations>,
}
