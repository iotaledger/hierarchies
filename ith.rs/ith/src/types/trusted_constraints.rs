use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use iota_sdk::types::base_types::{ObjectID, STD_OPTION_MODULE_NAME};
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::{Argument, Command};
use iota_sdk::types::{TypeTag, MOVE_STDLIB_PACKAGE_ID};
use move_core_types::ident_str;
use serde::{Deserialize, Serialize};

use super::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use super::{new_property_name, new_property_value_number, new_property_value_string};
use crate::utils::{self, deserialize_vec_map, deserialize_vec_set, MoveType};

/// Trusted property constraints for a federation
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

/// Trusted property constraint is a constraint that can be applied to a trusted property
/// to restrict the values that can be assigned to the property.
/// The constraint can be based on the property name, allowed values, or an expression.
/// The constraint can also have a time range in which the constraint is valid.
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

impl TrustedPropertyConstraint {
  pub fn new(property_name: impl Into<TrustedPropertyName>) -> Self {
    Self {
      property_name: property_name.into(),
      allowed_values: HashSet::new(),
      expression: None,
      allow_any: false,
      timespan: Timespan::default(),
    }
  }

  pub fn with_allowed_values(
    mut self,
    allowed_values: impl IntoIterator<Item = TrustedPropertyValue>,
  ) -> Self {
    self.allowed_values = allowed_values.into_iter().collect();
    self
  }

  pub fn with_expression(mut self, expression: TrustedPropertyExpression) -> Self {
    self.expression = Some(expression);
    self
  }

  pub fn with_timespan(mut self, timespan: Timespan) -> Self {
    self.timespan = timespan;
    self
  }
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

/// Trusted property expression is a constraint that can be applied to a trusted property
/// to restrict the values that can be assigned to the property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TrustedPropertyExpressionMove")]
pub enum TrustedPropertyExpression {
  StartsWith(String),
  EndsWith(String),
  Contains(String),
  GreaterThan(u64),
  LowerThan(u64),
}

impl MoveType for TrustedPropertyExpression {
  fn move_type(package: ObjectID) -> TypeTag {
    TypeTag::from_str(
      format!("{}::trusted_constraint::TrustedPropertyExpression", package).as_str(),
    )
    .expect("Failed to create type tag")
  }
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
struct TrustedPropertyExpressionMove {
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

impl MoveType for TrustedPropertyConstraint {
  fn move_type(package: ObjectID) -> TypeTag {
    TypeTag::from_str(
      format!("{}::trusted_constraint::TrustedPropertyConstraint", package).as_str(),
    )
    .expect("Failed to create type tag")
  }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Time-range for the constraint
pub struct Timespan {
  pub valid_from_ms: Option<u64>,
  pub valid_until_ms: Option<u64>,
}

/// Creates a new move type for a trusted property constraint
pub(crate) fn new_property_constraint(
  package_id: ObjectID,
  ptb: &mut ProgrammableTransactionBuilder,
  constraints: Vec<TrustedPropertyConstraint>,
) -> anyhow::Result<Argument> {
  let mut constraint_args = vec![];
  for constraint in constraints {
    let value_tag = TrustedPropertyValue::move_type(package_id);

    let property_names = new_property_name(constraint.property_name, ptb, package_id)?;

    let allow_any = ptb.pure(constraint.allow_any)?;

    let allowed_values = constraint
      .allowed_values
      .into_iter()
      .map(|value| match value {
        TrustedPropertyValue::Text(text) => {
          new_property_value_string(text.to_string(), ptb, package_id)
            .expect("failed to create new property value string")
        }
        TrustedPropertyValue::Number(number) => new_property_value_number(number, ptb, package_id)
          .expect("failed to create new property value number"),
      })
      .collect();

    let allowed_values =
      utils::create_vec_set_from_move_values(allowed_values, value_tag, ptb, package_id);

    let property_expression_tag = TrustedPropertyExpression::move_type(package_id);

    let expression = match constraint.expression {
      Some(expression) => {
        let string_tag =
          TypeTag::from_str(format!("{}::string::String", MOVE_STDLIB_PACKAGE_ID).as_str())?;

        let starts_with = match expression.as_starts_with() {
          Some(value) => utils::new_move_string(value, ptb)?,
          None => utils::option_to_move::<String>(None, string_tag.clone(), ptb)?,
        };

        let ends_with = match expression.as_ends_with() {
          Some(value) => utils::new_move_string(value, ptb)?,
          None => utils::option_to_move::<String>(None, string_tag.clone(), ptb)?,
        };

        let contains = match expression.as_contains() {
          Some(value) => utils::new_move_string(value, ptb)?,
          None => utils::option_to_move::<String>(None, string_tag.clone(), ptb)?,
        };

        let greater_than = utils::option_to_move(expression.as_greater_than(), TypeTag::U64, ptb)?;
        let lower_than = utils::option_to_move(expression.as_lower_than(), TypeTag::U64, ptb)?;

        let arg = ptb.programmable_move_call(
          package_id,
          ident_str!("trusted_constraint").into(),
          ident_str!("new_trusted_property_expression").into(),
          vec![],
          vec![starts_with, ends_with, contains, greater_than, lower_than],
        );

        ptb.programmable_move_call(
          MOVE_STDLIB_PACKAGE_ID,
          STD_OPTION_MODULE_NAME.into(),
          ident_str!("some").into(),
          vec![property_expression_tag],
          vec![arg],
        )
      }

      None => {
        utils::option_to_move::<TrustedPropertyConstraint>(None, property_expression_tag, ptb)?
      }
    };

    let constraint = ptb.programmable_move_call(
      package_id,
      ident_str!("trusted_constraint").into(),
      ident_str!("new_trusted_property_constraint").into(),
      vec![],
      vec![property_names, allowed_values, allow_any, expression],
    );
    constraint_args.push(constraint);
  }

  Ok(ptb.command(Command::MakeMoveVec(
    Some(TrustedPropertyConstraint::move_type(package_id)),
    constraint_args,
  )))
}
