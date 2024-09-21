use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrustedPropertyName {
  names: Vec<String>,
}

impl TrustedPropertyName {
  /// Create a new TrustedPropertyName
  pub fn new(names: Vec<String>) -> Self {
    Self { names }
  }

  pub fn names(&self) -> &Vec<String> {
    &self.names
  }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct TrustedPropertyValueMove {
  pub text: Option<String>,
  pub number: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[serde(try_from = "TrustedPropertyValueMove")]
pub enum TrustedPropertyValue {
  Text(String),
  Number(u64),
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
    let name = TrustedPropertyName::new(vec!["name".to_string(), "name2".to_string()]);

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
    let text = TrustedPropertyValue::Text("text".to_string());
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
