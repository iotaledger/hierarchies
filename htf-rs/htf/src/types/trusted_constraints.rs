use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::de::deserialize_vec_map;
use crate::de::deserialize_vec_set;

use super::trusted_property::TrustedPropertyName;
use super::trusted_property::TrustedPropertyValue;

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
  property_name: TrustedPropertyName,
  // allow only set of values
  #[serde(deserialize_with = "deserialize_vec_set")]
  allowed_values: HashSet<TrustedPropertyValue>,
  expression: Option<TrustedPropertyExpression>,
  // allow_any - takes a precedence over the allowed_values
  allow_any: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedPropertyExpression {
  starts_with: Option<String>,
  ends_with: Option<String>,
  contains: Option<String>,
  greater_than: Option<u64>,
  lower_than: Option<u64>,
}

impl TrustedPropertyConstraint {
  pub fn matches_constraint(&self, constraint: &TrustedPropertyConstraint) -> bool {
    if constraint.allow_any {
      return self.allow_any;
    }
    if constraint.expression.is_some() && self.expression.is_some() {
      return self.expression == constraint.expression;
    }

    self.allowed_values.is_superset(&constraint.allowed_values)
  }

  pub fn matches_property(&self, name: &TrustedPropertyName, value: &TrustedPropertyValue) -> bool {
    self.matches_name(name) && self.matches_value(value)
  }

  pub fn matches_name(&self, name: &TrustedPropertyName) -> bool {
    let len_constraint = self.property_name.names.len();
    let len_names = name.names.len();

    if len_constraint > len_names {
      return false;
    }

    self
      .property_name
      .names
      .iter()
      .zip(name.names.iter())
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

  pub fn to_map_of_constraints(
    constraints: Vec<TrustedPropertyConstraint>,
  ) -> HashMap<TrustedPropertyName, TrustedPropertyConstraint> {
    constraints
      .into_iter()
      .map(|constraint| (constraint.property_name.clone(), constraint))
      .collect()
  }

  pub fn matches_expression(exp: &TrustedPropertyExpression, value: &TrustedPropertyValue) -> bool {
    if let Some(ref starts_with) = exp.starts_with {
      if let Some(value_string) = &value.text {
        return value_string.starts_with(starts_with);
      }
    }

    if let Some(ref ends_with) = exp.ends_with {
      if let Some(value_string) = &value.text {
        return value_string.ends_with(ends_with);
      }
    }

    if let Some(ref contains) = exp.contains {
      if let Some(value_string) = &value.text {
        return value_string.contains(contains);
      }
    }

    if let Some(greater_than) = exp.greater_than {
      if let Some(value_number) = value.number {
        return value_number > greater_than;
      }
    }

    if let Some(lower_than) = exp.lower_than {
      if let Some(value_number) = value.number {
        return value_number < lower_than;
      }
    }

    false
  }
}
