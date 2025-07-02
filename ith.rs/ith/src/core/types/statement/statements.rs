use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{Statement, StatementName};
use crate::utils::deserialize_vec_map;

/// Statements for a federation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statements {
    #[serde(deserialize_with = "deserialize_vec_map")]
    pub data: HashMap<StatementName, Statement>,
}

impl Statements {
    pub fn contains_property(&self, statement_name: &StatementName) -> bool {
        self.data.contains_key(statement_name)
    }
}
