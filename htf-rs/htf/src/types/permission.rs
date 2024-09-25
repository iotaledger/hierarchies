use std::collections::HashMap;

use iota_sdk::types::id::{ID, UID};
use serde::{Deserialize, Serialize};

use super::trusted_constraints::TrustedPropertyConstraint;
use super::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use crate::utils::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
  pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
  pub id: UID,
  pub federation_id: ID,
  pub created_by: String,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub constraints: HashMap<TrustedPropertyName, TrustedPropertyConstraint>,
}

impl Permissions {
  pub fn are_values_permitted(
    &self,
    trusted_properties: &HashMap<TrustedPropertyName, TrustedPropertyValue>,
  ) -> bool {
    trusted_properties
      .iter()
      .all(|(property_name, property_value)| self.is_value_permitted(property_name, property_value))
  }

  pub fn is_value_permitted(
    &self,
    property_name: &TrustedPropertyName,
    property_value: &TrustedPropertyValue,
  ) -> bool {
    self
      .permissions
      .iter()
      .flat_map(|accreditation| accreditation.constraints.get(property_name))
      .any(|property_constraint| {
        property_constraint.matches_property(property_name, property_value)
      })
  }
}
