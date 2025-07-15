use std::str::FromStr;

use iota_interaction::ident_str;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::Argument;
use iota_interaction::types::TypeTag;
use iota_interaction::MoveType;
use serde::{Deserialize, Serialize};

/// StatementValue represents the value of a Statement
/// It can be either a text or a number
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum StatementValue {
    Text(String),
    Number(u64),
}

impl StatementValue {
    /// Converts the StatementValue to a ProgrammableTransactionBuilder argument
    pub(crate) fn to_ptb(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        package_id: ObjectID,
    ) -> anyhow::Result<Argument> {
        match self.clone() {
            StatementValue::Text(text) => new_statement_value_string(text, ptb, package_id),
            StatementValue::Number(number) => new_statement_value_number(number, ptb, package_id),
        }
    }
}

/// Creates a new move type for a Statement value string
pub(crate) fn new_statement_value_string(
    value: String,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let v = ptb.pure(value)?;
    Ok(ptb.programmable_move_call(
        package_id,
        ident_str!("statement_value").into(),
        ident_str!("new_statement_value_string").into(),
        vec![],
        vec![v],
    ))
}

/// Creates a new move type for a Statement value number
pub(crate) fn new_statement_value_number(
    value: u64,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let v = ptb.pure(value)?;
    Ok(ptb.programmable_move_call(
        package_id,
        ident_str!("statement_value").into(),
        ident_str!("new_statement_value_number").into(),
        vec![],
        vec![v],
    ))
}

impl MoveType for StatementValue {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{package}::statement_value::StatementValue").as_str())
            .expect("Failed to create type tag")
    }
}
