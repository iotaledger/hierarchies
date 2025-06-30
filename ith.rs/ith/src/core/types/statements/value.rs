use std::str::FromStr;

use iota_interaction::MoveType;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::Argument;
use iota_sdk::types::TypeTag;
use move_core_types::ident_str;
use serde::{Deserialize, Serialize};
/// StatementValue represents the value of a Statement
/// It can be either a text or a number
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum StatementValue {
    Text(String),
    Number(u64),
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
        TypeTag::from_str(format!("{}::statement_value::StatementValue", package).as_str())
            .expect("Failed to create type tag")
    }
}
