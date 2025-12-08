// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types for the Hierarchies protocol.

mod accreditation;
mod cap;
pub mod events;
pub mod property;
pub mod property_name;
pub mod property_shape;
pub mod property_value;
pub mod timespan;

use std::collections::HashMap;

pub use accreditation::*;
pub use cap::*;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::id::UID;
use serde::{Deserialize, Serialize};

use crate::core::types::property::FederationProperties;
use crate::utils::deserialize_vec_map;

/// Move package module names for Hierarchies smart contract interactions.
///
/// These constants define the module names used when calling functions
/// in the Hierarchies Move package deployed on the IOTA network.
pub mod move_names {
    /// The main Hierarchies package name
    pub const PACKAGE_NAME: &str = "hierarchies";
    /// Main module containing federation and core operations
    pub const MODULE_MAIN: &str = "main";
    /// Module for property-related operations
    pub const MODULE_PROPERTY: &str = "property";
    /// Module for property value operations
    pub const MODULE_VALUE: &str = "property_value";
    /// Module for property name operations
    pub const MODULE_NAME: &str = "property_name";
    /// Module for property shape operations
    pub const MODULE_SHAPE: &str = "property_shape";
    /// Utility module for common operations
    pub const MODULE_UTILS: &str = "utils";
}

/// Represents a federation. A federation is a group of entities that have agreed to work together
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Federation {
    pub id: UID,
    pub governance: Governance,
    pub root_authorities: Vec<RootAuthority>,
    pub revoked_root_authorities: Vec<ObjectID>,
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
    pub properties: FederationProperties,
    #[serde(deserialize_with = "deserialize_vec_map")]
    pub accreditations_to_accredit: HashMap<ObjectID, Accreditations>,
    #[serde(deserialize_with = "deserialize_vec_map")]
    pub accreditations_to_attest: HashMap<ObjectID, Accreditations>,
}
