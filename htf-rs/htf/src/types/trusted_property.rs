use serde::{Deserialize, Serialize};

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
