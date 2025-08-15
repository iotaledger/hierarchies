// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use iota_interaction::types::id::UID;
use serde::{Deserialize, Serialize};

use crate::core::types::property::FederationProperty;
use crate::core::types::property_name::PropertyName;
use crate::utils::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accreditations {
    pub accreditations: Vec<Accreditation>,
}

impl Accreditations {
    pub fn new(accreditations: Vec<Accreditation>) -> Self {
        Self { accreditations }
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Accreditation> {
        self.accreditations.iter()
    }
    pub fn len(&self) -> usize {
        self.accreditations.len()
    }
    pub fn is_empty(&self) -> bool {
        self.accreditations.is_empty()
    }
}

/// Represents an accreditation that can be granted to an account. An accreditation
/// consists of a set of properties that must be satisfied by the account in
/// order to be granted the accreditation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accreditation {
    pub id: UID,
    pub accredited_by: String,
    #[serde(deserialize_with = "deserialize_vec_map")]
    pub properties: HashMap<PropertyName, FederationProperty>,
}
