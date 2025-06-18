use std::collections::HashMap;

use iota_sdk::types::id::UID;
use serde::{Deserialize, Serialize};

use super::statement_name::StatementName;
use super::statements::Statement;
use crate::core::types::StatementValue;
use crate::utils::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accreditations {
    pub statements: Vec<Accreditation>,
}

/// Represents a statement that can be granted to an account. A statement
/// consists of a set of statements that must be satisfied by the accountaccreditedstatement in
/// order to be granted the statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accreditation {
    pub id: UID,
    pub accredited_by: String,
    #[serde(deserialize_with = "deserialize_vec_map")]
    pub statements: HashMap<StatementName, Statement>,
}

impl Accreditations {
    /// Checks if all the values in the provided `trusted_statements` map are
    /// permitted
    /// according to the statements defined in this `Accreditations` instance.
    pub fn are_statements_allowed(&self, trusted_statements: &HashMap<StatementName, StatementValue>) -> bool {
        trusted_statements
            .iter()
            .all(|(statement_name, property_value)| self.is_statement_allowed(statement_name, property_value))
    }

    /// Checks if the given `property_value` is permitted according to the
    /// statements
    /// defined in the `Accreditations` instance.
    pub fn is_statement_allowed(&self, statement_name: &StatementName, property_value: &StatementValue) -> bool {
        self.statements
            .iter()
            .flat_map(|accreditation| accreditation.statements.get(statement_name))
            .any(|property_statement| property_statement.matches_name_value(statement_name, property_value))
    }
}
