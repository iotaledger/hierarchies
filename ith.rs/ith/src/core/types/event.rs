// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_interaction::types::base_types::ObjectID;
use serde::{Deserialize, Serialize};

/// An event that can be emitted by the ITH.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event<D> {
    pub data: D,
}

/// An event that is emitted when a new federation is created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationCreatedEvent {
    pub federation_address: ObjectID,
}
