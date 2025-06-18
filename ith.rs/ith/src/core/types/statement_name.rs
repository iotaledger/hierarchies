use std::str::FromStr;

use iota_interaction::MoveType;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::Argument;
use iota_sdk::types::TypeTag;
use move_core_types::ident_str;
use serde::{Deserialize, Serialize};

/// StatementName represents the name of a Statement
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatementName {
    names: Vec<String>,
}

impl<D> From<D> for StatementName
where
    D: Into<String>,
{
    fn from(name: D) -> Self {
        Self {
            names: vec![name.into()],
        }
    }
}

impl StatementName {
    /// Create a new StatementName
    pub fn new<D>(names: impl IntoIterator<Item = D>) -> Self
    where
        D: Into<String>,
    {
        Self {
            names: names.into_iter().map(Into::into).collect(),
        }
    }

    pub fn names(&self) -> &Vec<String> {
        &self.names
    }
}

impl MoveType for StatementName {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{}::trusted_statement::StatementName", package).as_str())
            .expect("Failed to create type tag")
    }
}

/// Creates a new move type for a Statement name
pub(crate) fn newstatement_name(
    name: StatementName,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let names = ptb.pure(name.names())?;
    let statement_names: Argument = ptb.programmable_move_call(
        package_id,
        ident_str!("trusted_statement").into(),
        ident_str!("newstatement_name_from_vector").into(),
        vec![],
        vec![names],
    );

    Ok(statement_names)
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct StatementValueMove {
    pub text: Option<String>,
    pub number: Option<u64>,
}

/// StatementValue represents the value of a Statement
/// It can be either a text or a number
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[serde(try_from = "StatementValueMove")]
pub enum StatementValue {
    Text(String),
    Number(u64),
}

/// Creates a new move type for a Statement value string
pub(crate) fn new_property_value_string(
    value: String,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let v = ptb.pure(value)?;
    Ok(ptb.programmable_move_call(
        package_id,
        ident_str!("statement_value").into(),
        ident_str!("new_property_value_string").into(),
        vec![],
        vec![v],
    ))
}

/// Creates a new move type for a Statement value number
pub(crate) fn new_property_value_number(
    value: u64,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let v = ptb.pure(value)?;
    Ok(ptb.programmable_move_call(
        package_id,
        ident_str!("statement_value").into(),
        ident_str!("new_property_value_number").into(),
        vec![],
        vec![v],
    ))
}

impl MoveType for StatementValue {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{}::trusted_statement::StatementValue", package).as_str())
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

impl TryFrom<StatementValueMove> for StatementValue {
    type Error = &'static str;

    fn try_from(value: StatementValueMove) -> Result<Self, Self::Error> {
        match (value.text, value.number) {
            (Some(text), None) => Ok(StatementValue::Text(text)),
            (None, Some(number)) => Ok(StatementValue::Number(number)),
            _ => Err("Invalid StatementValue: must have either text or number, not both or neither"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_trusted_statement_name() {
        let name = StatementName::new(["name", "name2"]);

        let json = json!({
          "names": ["name", "name2"]
        });

        assert_eq!(serde_json::to_value(&name).unwrap(), json);
        assert_eq!(serde_json::from_value::<StatementName>(json).unwrap(), name);
    }

    #[test]
    fn test_trusted_statement_value() {
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
