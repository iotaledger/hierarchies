mod name;
mod statements;
mod value;
mod value_condition;

pub use name::*;
pub use statements::*;
pub use value::*;
pub use value_condition::*;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::utils::deserialize_vec_set;

/// Statement is a statement that can be applied to a Statement
/// to restrict the values that can be assigned to the property.
/// The statement can be based on the property name, allowed values, or an expression.
/// The statement can also have a time range in which the statement is valid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// The evaluation order: allow_any => expression => allowed_values
pub struct Statement {
  pub statement_name: StatementName,
  // allow only set of values
  #[serde(deserialize_with = "deserialize_vec_set")]
  pub allowed_values: HashSet<StatementValue>,
  pub condition: Option<StatementValueCondition>,
  // allow_any - takes a precedence over the allowed_values
  pub allow_any: bool,
  pub timespan: Timespan,
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

  pub fn with_allowed_values(
    mut self,
    allowed_values: impl IntoIterator<Item = StatementValue>,
  ) -> Self {
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

impl Statement {
  pub fn matches_name_value(&self, name: &StatementName, value: &StatementValue) -> bool {
    self.matches_name(name) && self.matches_value(value)
  }

  pub fn matches_name(&self, name: &StatementName) -> bool {
    let len_statement = self.statement_name.names().len();
    let len_names = name.names().len();

    if len_statement > len_names {
      return false;
    }

    self
      .statement_name
      .names()
      .iter()
      .zip(name.names().iter())
      .all(|(a, b)| a == b)
  }

  pub fn matches_value(&self, value: &StatementValue) -> bool {
    if self.allow_any {
      return true;
    }
    if let Some(ref expression) = self.condition {
      if Self::matches_expression(expression, value) {
        return true;
      }
    }
    self.allowed_values.contains(value)
  }

  pub fn matches_expression(exp: &StatementValueCondition, value: &StatementValue) -> bool {
    match exp {
      StatementValueCondition::StartsWith(prefix) => {
        if let StatementValue::Text(text) = value {
          text.starts_with(prefix)
        } else {
          false
        }
      }
      StatementValueCondition::EndsWith(suffix) => {
        if let StatementValue::Text(text) = value {
          text.ends_with(suffix)
        } else {
          false
        }
      }
      StatementValueCondition::Contains(substring) => {
        if let StatementValue::Text(text) = value {
          text.contains(substring)
        } else {
          false
        }
      }
      StatementValueCondition::GreaterThan(num) => {
        if let StatementValue::Number(value) = value {
          value > num
        } else {
          false
        }
      }
      StatementValueCondition::LowerThan(num) => {
        if let StatementValue::Number(value) = value {
          value < num
        } else {
          false
        }
      }
    }
  }
}
