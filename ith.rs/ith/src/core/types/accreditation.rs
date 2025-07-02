use std::collections::HashMap;

use iota_sdk::types::id::UID;
use serde::{Deserialize, Serialize};

use crate::core::types::statements::name::StatementName;
use crate::core::types::statements::Statement;
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
