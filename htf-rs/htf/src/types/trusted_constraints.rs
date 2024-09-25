use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use crate::utils::{deserialize_vec_map, deserialize_vec_set};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedPropertyConstraints {
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub data: HashMap<TrustedPropertyName, TrustedPropertyConstraint>,
}

impl TrustedPropertyConstraints {
  pub fn contains_property(&self, property_name: &TrustedPropertyName) -> bool {
    self.data.contains_key(property_name)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// The evaluation order: allow_any => expression => allowed_values
pub struct TrustedPropertyConstraint {
  pub property_name: TrustedPropertyName,
  // allow only set of values
  #[serde(deserialize_with = "deserialize_vec_set")]
  pub allowed_values: HashSet<TrustedPropertyValue>,
  pub expression: Option<TrustedPropertyExpression>,
  // allow_any - takes a precedence over the allowed_values
  pub allow_any: bool,
  pub timespan: Timespan,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Timerange for the constraint
pub struct Timespan {
  pub valid_from_ms: Option<u64>,
  pub valid_until_ms: Option<u64>,
}

impl TrustedPropertyConstraint {
  pub fn matches_property(&self, name: &TrustedPropertyName, value: &TrustedPropertyValue) -> bool {
    self.matches_name(name) && self.matches_value(value)
  }

  pub fn matches_name(&self, name: &TrustedPropertyName) -> bool {
    let len_constraint = self.property_name.names().len();
    let len_names = name.names().len();

    if len_constraint > len_names {
      return false;
    }

    self
      .property_name
      .names()
      .iter()
      .zip(name.names().iter())
      .all(|(a, b)| a == b)
  }

  pub fn matches_value(&self, value: &TrustedPropertyValue) -> bool {
    if self.allow_any {
      return true;
    }
    if let Some(ref expression) = self.expression {
      if Self::matches_expression(expression, value) {
        return true;
      }
    }
    self.allowed_values.contains(value)
  }

  pub fn matches_expression(exp: &TrustedPropertyExpression, value: &TrustedPropertyValue) -> bool {
    match exp {
      TrustedPropertyExpression::StartsWith(prefix) => {
        if let TrustedPropertyValue::Text(text) = value {
          text.starts_with(prefix)
        } else {
          false
        }
      }
      TrustedPropertyExpression::EndsWith(suffix) => {
        if let TrustedPropertyValue::Text(text) = value {
          text.ends_with(suffix)
        } else {
          false
        }
      }
      TrustedPropertyExpression::Contains(substring) => {
        if let TrustedPropertyValue::Text(text) = value {
          text.contains(substring)
        } else {
          false
        }
      }
      TrustedPropertyExpression::GreaterThan(num) => {
        if let TrustedPropertyValue::Number(value) = value {
          value > num
        } else {
          false
        }
      }
      TrustedPropertyExpression::LowerThan(num) => {
        if let TrustedPropertyValue::Number(value) = value {
          value < num
        } else {
          false
        }
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TrustedPropertyExpressionMove")]
pub enum TrustedPropertyExpression {
  StartsWith(String),
  EndsWith(String),
  Contains(String),
  GreaterThan(u64),
  LowerThan(u64),
}

impl TrustedPropertyExpression {
  pub fn as_starts_with(&self) -> Option<String> {
    match self {
      TrustedPropertyExpression::StartsWith(value) => Some(value.clone()),
      _ => None,
    }
  }
  pub fn as_ends_with(&self) -> Option<String> {
    match self {
      TrustedPropertyExpression::EndsWith(value) => Some(value.clone()),
      _ => None,
    }
  }
  pub fn as_contains(&self) -> Option<String> {
    match self {
      TrustedPropertyExpression::Contains(value) => Some(value.clone()),
      _ => None,
    }
  }
  pub fn as_greater_than(&self) -> Option<u64> {
    match self {
      TrustedPropertyExpression::GreaterThan(value) => Some(*value),
      _ => None,
    }
  }
  pub fn as_lower_than(&self) -> Option<u64> {
    match self {
      TrustedPropertyExpression::LowerThan(value) => Some(*value),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedPropertyExpressionMove {
  starts_with: Option<String>,
  ends_with: Option<String>,
  contains: Option<String>,
  greater_than: Option<u64>,
  lower_than: Option<u64>,
}

impl TryFrom<TrustedPropertyExpressionMove> for TrustedPropertyExpression {
  type Error = &'static str;

  fn try_from(value: TrustedPropertyExpressionMove) -> Result<Self, Self::Error> {
    match (value.starts_with, value.ends_with, value.contains, value.greater_than, value.lower_than) {
      (Some(starts_with), None, None, None, None) => Ok(TrustedPropertyExpression::StartsWith(starts_with)),
      (None, Some(ends_with), None, None, None) => Ok(TrustedPropertyExpression::EndsWith(ends_with)),
      (None, None, Some(contains), None, None) => Ok(TrustedPropertyExpression::Contains(contains)),
      (None, None, None, Some(greater_than), None) => Ok(TrustedPropertyExpression::GreaterThan(greater_than)),
      (None, None, None, None, Some(lower_than)) => Ok(TrustedPropertyExpression::LowerThan(lower_than)),
      _ => Err("Invalid TrustedPropertyExpression: must have either starts_with, ends_with, contains, greater_than or lower_than"),
    }
  }
}
