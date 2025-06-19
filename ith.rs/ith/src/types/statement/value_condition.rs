use std::str::FromStr;

use iota_sdk::types::{
  base_types::{ObjectID, STD_OPTION_MODULE_NAME},
  programmable_transaction_builder::ProgrammableTransactionBuilder,
  transaction::{Argument, Command},
  TypeTag, MOVE_STDLIB_PACKAGE_ID,
};
use move_core_types::ident_str;
use serde::{Deserialize, Serialize};

use crate::{
  types::{
    new_statement_name, Statement, StatementValue,
    {new_statement_value_number, new_statement_value_string},
  },
  utils::{self, MoveType},
};

/// Trusted property expression is a statement that can be applied to a Statement
/// to restrict the values that can be assigned to the property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// #[serde(try_from = "StatementValueConditionMove")]
pub enum StatementValueCondition {
  StartsWith(String),
  EndsWith(String),
  Contains(String),
  GreaterThan(u64),
  LowerThan(u64),
}

impl MoveType for StatementValueCondition {
  fn move_type(package: ObjectID) -> TypeTag {
    TypeTag::from_str(format!("{}::statement_condition::StatementValueCondition", package).as_str())
      .expect("Failed to create type tag")
  }
}
impl StatementValueCondition {
  pub fn as_starts_with(&self) -> Option<String> {
    match self {
      StatementValueCondition::StartsWith(value) => Some(value.clone()),
      _ => None,
    }
  }
  pub fn as_ends_with(&self) -> Option<String> {
    match self {
      StatementValueCondition::EndsWith(value) => Some(value.clone()),
      _ => None,
    }
  }
  pub fn as_contains(&self) -> Option<String> {
    match self {
      StatementValueCondition::Contains(value) => Some(value.clone()),
      _ => None,
    }
  }
  pub fn as_greater_than(&self) -> Option<u64> {
    match self {
      StatementValueCondition::GreaterThan(value) => Some(*value),
      _ => None,
    }
  }
  pub fn as_lower_than(&self) -> Option<u64> {
    match self {
      StatementValueCondition::LowerThan(value) => Some(*value),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct StatementValueConditionMove {
  starts_with: Option<String>,
  ends_with: Option<String>,
  contains: Option<String>,
  greater_than: Option<u64>,
  lower_than: Option<u64>,
}

impl TryFrom<StatementValueConditionMove> for StatementValueCondition {
  type Error = &'static str;

  fn try_from(value: StatementValueConditionMove) -> Result<Self, Self::Error> {
    match (value.starts_with, value.ends_with, value.contains, value.greater_than, value.lower_than) {
      (Some(starts_with), None, None, None, None) => Ok(StatementValueCondition::StartsWith(starts_with)),
      (None, Some(ends_with), None, None, None) => Ok(StatementValueCondition::EndsWith(ends_with)),
      (None, None, Some(contains), None, None) => Ok(StatementValueCondition::Contains(contains)),
      (None, None, None, Some(greater_than), None) => Ok(StatementValueCondition::GreaterThan(greater_than)),
      (None, None, None, None, Some(lower_than)) => Ok(StatementValueCondition::LowerThan(lower_than)),
      _ => Err("Invalid StatementValueCondition: must have either starts_with, ends_with, contains, greater_than or lower_than"),
    }
  }
}

impl MoveType for Statement {
  fn move_type(package: ObjectID) -> TypeTag {
    TypeTag::from_str(format!("{}::statement::Statement", package).as_str())
      .expect("Failed to create type tag")
  }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Time-range for the statement
pub struct Timespan {
  pub valid_from_ms: Option<u64>,
  pub valid_until_ms: Option<u64>,
}

/// Creates a new move type for a Statement statement
pub(crate) fn new_property_statement(
  package_id: ObjectID,
  ptb: &mut ProgrammableTransactionBuilder,
  statements: Vec<Statement>,
) -> anyhow::Result<Argument> {
  let mut statement_args = vec![];
  for statement in statements {
    let value_tag = StatementValue::move_type(package_id);

    let statement_names = new_statement_name(statement.statement_name, ptb, package_id)?;

    let allow_any = ptb.pure(statement.allow_any)?;

    let allowed_values = statement
      .allowed_values
      .into_iter()
      .map(|value| match value {
        StatementValue::Text(text) => new_statement_value_string(text.to_string(), ptb, package_id)
          .expect("failed to create new property value string"),
        StatementValue::Number(number) => new_statement_value_number(number, ptb, package_id)
          .expect("failed to create new property value number"),
      })
      .collect();

    let allowed_values =
      utils::create_vec_set_from_move_values(allowed_values, value_tag, ptb, package_id);

    let property_expression_tag = StatementValueCondition::move_type(package_id);

    let expression = match statement.condition {
      Some(expression) => {
        let (condition_arg, constructor_name) = match expression {
          StatementValueCondition::StartsWith(condition) => (
            utils::new_move_string(condition, ptb)?,
            ident_str!("new_condition_starts_with").into(),
          ),
          StatementValueCondition::EndsWith(condition) => (
            utils::new_move_string(condition, ptb)?,
            ident_str!("new_condition_ends_with").into(),
          ),
          StatementValueCondition::Contains(condition) => (
            utils::new_move_string(condition, ptb)?,
            ident_str!("new_condition_contains").into(),
          ),
          StatementValueCondition::GreaterThan(value) => (
            ptb.pure(value)?,
            ident_str!("new_condition_greater_than").into(),
          ),
          StatementValueCondition::LowerThan(value) => (
            ptb.pure(value)?,
            ident_str!("new_condition_lower_than").into(),
          ),
        };

        let arg = ptb.programmable_move_call(
          package_id,
          ident_str!("statement_condition").into(),
          constructor_name,
          vec![],
          vec![condition_arg],
        );

        ptb.programmable_move_call(
          MOVE_STDLIB_PACKAGE_ID,
          STD_OPTION_MODULE_NAME.into(),
          ident_str!("some").into(),
          vec![property_expression_tag],
          vec![arg],
        )
      }

      None => utils::option_to_move::<Statement>(None, property_expression_tag, ptb)?,
    };

    let statement = ptb.programmable_move_call(
      package_id,
      ident_str!("statement").into(),
      ident_str!("new_statement").into(),
      vec![],
      vec![statement_names, allowed_values, allow_any, expression],
    );
    statement_args.push(statement);
  }

  Ok(ptb.command(Command::MakeMoveVec(
    Some(Statement::move_type(package_id)),
    statement_args,
  )))
}
