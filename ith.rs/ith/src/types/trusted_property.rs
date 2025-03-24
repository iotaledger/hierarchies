use std::str::FromStr;

use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::Argument;
use iota_sdk::types::TypeTag;
use move_core_types::ident_str;
use serde::{Deserialize, Serialize};

use crate::utils::MoveType;

/// TrustedPropertyName represents the name of a trusted property
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrustedPropertyName {
  names: Vec<String>,
}

impl<D> From<D> for TrustedPropertyName
where
  D: Into<String>,
{
  fn from(name: D) -> Self {
    Self {
      names: vec![name.into()],
    }
  }
}

impl TrustedPropertyName {
  /// Create a new TrustedPropertyName
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

impl MoveType for TrustedPropertyName {
  fn move_type(package: ObjectID) -> TypeTag {
    TypeTag::from_str(format!("{}::trusted_property::TrustedPropertyName", package).as_str())
      .expect("Failed to create type tag")
  }
}

/// Creates a new move type for a trusted property name
pub(crate) fn new_property_name(
  name: TrustedPropertyName,
  ptb: &mut ProgrammableTransactionBuilder,
  package_id: ObjectID,
) -> anyhow::Result<Argument> {
  let names = ptb.pure(name.names())?;
  let property_names: Argument = ptb.programmable_move_call(
    package_id,
    ident_str!("trusted_property").into(),
    ident_str!("new_property_name_from_vector").into(),
    vec![],
    vec![names],
  );

  Ok(property_names)
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct TrustedPropertyValueMove {
  pub text: Option<String>,
  pub number: Option<u64>,
}

/// TrustedPropertyValue represents the value of a trusted property
/// It can be either a text or a number
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[serde(try_from = "TrustedPropertyValueMove")]
pub enum TrustedPropertyValue {
  Text(String),
  Number(u64),
}

/// Creates a new move type for a trusted property value string
pub(crate) fn new_property_value_string(
  value: String,
  ptb: &mut ProgrammableTransactionBuilder,
  package_id: ObjectID,
) -> anyhow::Result<Argument> {
  let v = ptb.pure(value)?;
  Ok(ptb.programmable_move_call(
    package_id,
    ident_str!("trusted_property").into(),
    ident_str!("new_property_value_string").into(),
    vec![],
    vec![v],
  ))
}

/// Creates a new move type for a trusted property value number
pub(crate) fn new_property_value_number(
  value: u64,
  ptb: &mut ProgrammableTransactionBuilder,
  package_id: ObjectID,
) -> anyhow::Result<Argument> {
  let v = ptb.pure(value)?;
  Ok(ptb.programmable_move_call(
    package_id,
    ident_str!("trusted_property").into(),
    ident_str!("new_property_value_number").into(),
    vec![],
    vec![v],
  ))
}

impl MoveType for TrustedPropertyValue {
  fn move_type(package: ObjectID) -> TypeTag {
    TypeTag::from_str(format!("{}::trusted_property::TrustedPropertyValue", package).as_str())
      .expect("Failed to create type tag")
  }
}

impl From<String> for TrustedPropertyValue {
  fn from(text: String) -> Self {
    Self::Text(text)
  }
}

impl From<&str> for TrustedPropertyValue {
  fn from(text: &str) -> Self {
    Self::Text(text.to_string())
  }
}

impl From<u64> for TrustedPropertyValue {
  fn from(number: u64) -> Self {
    Self::Number(number)
  }
}

impl TryFrom<TrustedPropertyValueMove> for TrustedPropertyValue {
  type Error = &'static str;

  fn try_from(value: TrustedPropertyValueMove) -> Result<Self, Self::Error> {
    match (value.text, value.number) {
      (Some(text), None) => Ok(TrustedPropertyValue::Text(text)),
      (None, Some(number)) => Ok(TrustedPropertyValue::Number(number)),
      _ => {
        Err("Invalid TrustedPropertyValue: must have either text or number, not both or neither")
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_trusted_property_name() {
    let name = TrustedPropertyName::new(["name", "name2"]);

    let json = json!({
      "names": ["name", "name2"]
    });

    assert_eq!(serde_json::to_value(&name).unwrap(), json);
    assert_eq!(
      serde_json::from_value::<TrustedPropertyName>(json).unwrap(),
      name
    );
  }

  #[test]
  fn test_trusted_property_value() {
    let text = TrustedPropertyValue::from("text");
    let number = TrustedPropertyValue::Number(42);

    let json_text = json!({
      "text": "text"
    });

    let json_number = json!({
      "number": 42
    });

    assert_eq!(serde_json::to_value(&text).unwrap(), json_text);
    assert_eq!(
      serde_json::from_value::<TrustedPropertyValue>(json_text).unwrap(),
      text
    );

    assert_eq!(serde_json::to_value(&number).unwrap(), json_number);
    assert_eq!(
      serde_json::from_value::<TrustedPropertyValue>(json_number).unwrap(),
      number
    );
  }
}
