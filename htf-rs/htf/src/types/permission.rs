use std::collections::HashMap;

use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::id::{ID, UID};
use serde::{Deserialize, Serialize};

use super::trusted_constraints::TrustedPropertyConstraint;
use super::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use crate::utils::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub(crate) attestations: HashMap<ObjectID, Vec<PermissionToAttest>>,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub(crate) permissions_to_accredit: HashMap<ObjectID, Vec<PermissionToAccredit>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionsToAccredit {
  pub permissions: Vec<PermissionToAccredit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionToAccredit {
  id: UID,
  federation_id: ID,
  created_by: String,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub constraints: HashMap<TrustedPropertyName, TrustedPropertyConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionsToAttest {
  pub permissions: Vec<PermissionToAttest>,
}

impl PermissionsToAttest {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// PermissionToAttest can be created only by the HTF module
pub struct PermissionToAttest {
  id: UID,
  federation_id: ObjectID,
  created_by: String,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub constraints: HashMap<TrustedPropertyName, TrustedPropertyConstraint>,
}
