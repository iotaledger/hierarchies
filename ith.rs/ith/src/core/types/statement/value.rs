use std::str::FromStr;

use iota_interaction::MoveType;
use iota_sdk::types::{
    base_types::ObjectID, programmable_transaction_builder::ProgrammableTransactionBuilder, transaction::Argument,
    TypeTag,
};
use move_core_types::ident_str;
use serde::{Deserialize, Serialize};

use crate::core::operations::move_names;

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
        ident_str!(move_names::MODULE_VALUE).into(),
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
        ident_str!(move_names::MODULE_VALUE).into(),
        ident_str!("new_statement_value_number").into(),
        vec![],
        vec![v],
    ))
}

impl MoveType for StatementValue {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{}::{}::StatementValue", package, move_names::MODULE_VALUE).as_str())
            .expect("Failed to create type tag")
    }
}

impl From<String> for StatementValue {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for StatementValue {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<u64> for StatementValue {
    fn from(number: u64) -> Self {
        Self::Number(number)
    }
}

// impl TryFrom<StatementValueMove> for StatementValue {
//     type Error = &'static str;

//     fn try_from(value: StatementValueMove) -> Result<Self, Self::Error> {
//         match (value.text, value.number) {
//             (Some(text), None) => Ok(StatementValue::Text(text)),
//             (None, Some(number)) => Ok(StatementValue::Number(number)),
//             _ => Err("Invalid StatementValue: must have either text or number, not both or neither"),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_statement_value() {
        let text = StatementValue::from("text");
        let number = StatementValue::Number(42);

        let json_text = json!({
          "text": "text"
        });

        let json_number = json!({
          "number": 42
        });

        assert_eq!(serde_json::to_value(&text).unwrap(), json_text);
        assert_eq!(serde_json::from_value::<StatementValue>(json_text).unwrap(), text);

        assert_eq!(serde_json::to_value(&number).unwrap(), json_number);
        assert_eq!(serde_json::from_value::<StatementValue>(json_number).unwrap(), number);
    }
}
