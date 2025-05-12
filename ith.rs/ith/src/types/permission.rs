use std::collections::HashMap;

use iota_sdk::types::id::{ID, UID};
use serde::{Deserialize, Serialize};

use super::statement::{StatementName, StatementValue};
use super::trusted_constraints::Statement;
use crate::utils::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accreditations {
  pub permissions: Vec<Accreditation>,
}

/// Represents a permission that can be granted to an account. A permission
/// consists of a set of constraints that must be satisfied by the accountaccreditedstatement in
/// order to be granted the permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accreditation {
  pub id: UID,
  pub federation_id: ID,
  pub accredited_by: String,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub constraints: HashMap<StatementName, Statement>,
}

impl Accreditations {
  /// Checks if all the values in the provided `trusted_statements` map are
  /// permitted
  /// according to the permissions defined in this `Accreditations` instance.
  pub fn are_statements_allowed(
    &self,
    trusted_statements: &HashMap<StatementName, StatementValue>,
  ) -> bool {
    trusted_statements
      .iter()
      .all(|(statement_name, property_value)| {
        self.is_statement_allowed(statement_name, property_value)
      })
  }

  /// Checks if the given `property_value` is permitted according to the
  /// constraints
  /// defined in the `Accreditations` instance.
  pub fn is_statement_allowed(
    &self,
    statement_name: &StatementName,
    property_value: &StatementValue,
  ) -> bool {
    self
      .permissions
      .iter()
      .flat_map(|accreditation| accreditation.constraints.get(statement_name))
      .any(|property_constraint| {
        property_constraint.matches_name_value(statement_name, property_value)
      })
  }
}
