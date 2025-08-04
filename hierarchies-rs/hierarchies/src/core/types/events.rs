// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Events
//!
//! This module provides event types for the Hierarchies (IOTA Trust Hierarchy) module.

use iota_interaction::types::base_types::ObjectID;
use serde::{Deserialize, Serialize};

use crate::core::types::statements::name::StatementName;

/// Event emitted when a new federation is created
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationCreatedEvent {
    pub federation_address: ObjectID,
}

/// Event emitted when a statement is added to the federation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementAddedEvent {
    pub federation_address: ObjectID,
    pub statement_name: StatementName,
    pub allow_any: bool,
}

/// Event emitted when a statement is revoked
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementRevokedEvent {
    pub federation_address: ObjectID,
    pub statement_name: StatementName,
    pub valid_to_ms: u64,
}

/// Event emitted when a root authority is added
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthorityAddedEvent {
    pub federation_address: ObjectID,
    pub account_id: ObjectID,
}

/// Event emitted when a root authority is revoked
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthorityRevokedEvent {
    pub federation_address: ObjectID,
    pub account_id: ObjectID,
}

/// Event emitted when accreditation to accredit is created
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccreditationToAccreditCreatedEvent {
    pub federation_address: ObjectID,
    pub receiver: ObjectID,
    pub accreditor: ObjectID,
}

/// Event emitted when accreditation to attest is created
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccreditationToAttestCreatedEvent {
    pub federation_address: ObjectID,
    pub receiver: ObjectID,
    pub accreditor: ObjectID,
}

/// Event emitted when accreditation to attest is revoked
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccreditationToAttestRevokedEvent {
    pub federation_address: ObjectID,
    pub entity_id: ObjectID,
    pub permission_id: ObjectID,
    pub revoker: ObjectID,
}

/// Event emitted when accreditation to accredit is revoked
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccreditationToAccreditRevokedEvent {
    pub federation_address: ObjectID,
    pub entity_id: ObjectID,
    pub permission_id: ObjectID,
    pub revoker: ObjectID,
}
