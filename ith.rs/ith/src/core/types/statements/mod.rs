pub mod condition;
pub mod name;
pub mod value;

use std::collections::{HashMap, HashSet};

use iota_sdk::types::collection_types::{VecMap, VecSet};

use crate::core::types::statements::condition::StatementValueCondition;
use crate::core::types::statements::name::StatementName;
use crate::core::types::statements::value::StatementValue;
use crate::core::types::timespan::Timespan;
use crate::utils::{deserialize_vec_map, deserialize_vec_set};
use serde::{Deserialize, Serialize};

// Statements is a struct that contains a map of StatementName to Statement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statements {
    #[serde(deserialize_with = "deserialize_vec_map")]
    data: HashMap<StatementName, Statement>,
}

// The evaluation order: allow_any => condition => allowed_values
// The evaluation order is determined by the possible size of the set of values
// that match the condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statement {
    statement_name: StatementName,
    // allow only values that are in the set
    #[serde(deserialize_with = "deserialize_vec_set")]
    allowed_values: HashSet<StatementValue>,
    // Allow only values that match the condition.
    condition: Option<StatementValueCondition>,
    // If true, the statement is not applied, any value is allowed
    allow_any: bool,
    // The time span of the statement
    timespan: Timespan,
}

impl Statement {
    pub fn new(statement_name: impl Into<StatementName>) -> Self {
        Self {
            statement_name: statement_name.into(),
            allowed_values: HashSet::new(),
            condition: None,
            allow_any: false,
            timespan: Timespan::default(),
        }
    }

    pub fn with_allowed_values(mut self, allowed_values: impl IntoIterator<Item = StatementValue>) -> Self {
        self.allowed_values = allowed_values.into_iter().collect();
        self
    }

    pub fn with_expression(mut self, expression: StatementValueCondition) -> Self {
        self.condition = Some(expression);
        self
    }

    pub fn with_timespan(mut self, timespan: Timespan) -> Self {
        self.timespan = timespan;
        self
    }
}
