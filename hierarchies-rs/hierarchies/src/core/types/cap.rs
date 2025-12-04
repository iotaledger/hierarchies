// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Capabilities
//!
//! This module provides capability types for the Hierarchies (IOTA Trust Hierarchy)
//! module.

use std::str::FromStr;

use iota_interaction::MoveType;
use iota_interaction::move_types::language_storage::TypeTag;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::id::UID;
use serde::{Deserialize, Serialize};

use super::move_names;

/// Capability for root authority operations.
///
/// This capability grants full administrative access to a federation,
/// including the ability to add/remove other root authorities and manage properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthorityCap {
    pub id: UID,
    pub federation_id: ObjectID,
    pub account_id: ObjectID,
}

impl MoveType for RootAuthorityCap {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{package}::{}::RootAuthorityCap", move_names::MODULE_MAIN).as_str())
            .expect("Failed to create type tag")
    }
}

/// Capability for accreditation operations.
///
/// This capability grants the ability to delegate accreditation and attestation rights
/// to other accounts within a federation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccreditCap {
    pub id: UID,
    pub federation_id: ObjectID,
}

impl MoveType for AccreditCap {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{package}::{}::AccreditCap", move_names::MODULE_MAIN).as_str())
            .expect("Failed to create type tag")
    }
}
