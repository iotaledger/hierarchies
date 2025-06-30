pub mod condition;
pub mod name;
pub mod value;

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use iota_interaction::MoveType;
use iota_sdk::types::base_types::{ObjectID, STD_OPTION_MODULE_NAME};
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::{Argument, Command};
use iota_sdk::types::{TypeTag, MOVE_STDLIB_PACKAGE_ID};
use move_core_types::ident_str;

use crate::core::types::statements::condition::StatementValueCondition;
use crate::core::types::statements::name::{new_statement_name, StatementName};
use crate::core::types::statements::value::{new_statement_value_number, new_statement_value_string, StatementValue};
use crate::core::types::timespan::Timespan;
use crate::utils::{self, deserialize_vec_map, deserialize_vec_set};
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

impl MoveType for Statement {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{}::statement::Statement", package).as_str()).expect("Failed to create type tag")
    }
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

        let allowed_values = utils::create_vec_set_from_move_values(allowed_values, value_tag, ptb, package_id);

        let property_expression_tag = StatementValueCondition::move_type(package_id);

        let expression = match statement.condition {
            Some(expression) => {
                let expression = expression.into_ptb(ptb, package_id)?;
                utils::option_to_move(Some(expression), property_expression_tag, ptb)?
            }

            None => utils::option_to_move::<Statement>(None, property_expression_tag, ptb)?,
        };

        let statement = ptb.programmable_move_call(
            package_id,
            ident_str!("trusted_statement").into(),
            ident_str!("new_trusted_statement").into(),
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
