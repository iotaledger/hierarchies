use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use fastcrypto::ed25519::Ed25519PublicKey;
use fastcrypto::traits::ToFromBytes;
use iota_sdk::types::base_types::{
  IotaAddress, ObjectID, STD_OPTION_MODULE_NAME, STD_UTF8_MODULE_NAME,
};
use iota_sdk::types::collection_types::{VecMap, VecSet};
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::{Argument, Command};
use iota_sdk::types::{TypeTag, MOVE_STDLIB_PACKAGE_ID};
use move_core_types::ident_str;
use serde::{Deserialize, Deserializer, Serialize};

/// Trait to create a MoveType for move objects and types
pub trait MoveType {
  fn move_type(package: ObjectID) -> TypeTag;
}

pub fn convert_to_address(sender_public_key: &[u8]) -> anyhow::Result<IotaAddress> {
  let public_key = Ed25519PublicKey::from_bytes(sender_public_key).map_err(|err| {
    anyhow::anyhow!(format!(
      "could not parse public key to Ed25519 public key; {err}"
    ))
  })?;

  Ok(IotaAddress::from(&public_key))
}

pub(crate) fn deserialize_vec_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
  D: Deserializer<'de>,
  K: Deserialize<'de> + Eq + Hash + Debug,
  V: Deserialize<'de> + Debug,
{
  let vec_map = VecMap::<K, V>::deserialize(deserializer)?;
  Ok(
    vec_map
      .contents
      .into_iter()
      .map(|entry| (entry.key, entry.value))
      .collect(),
  )
}

pub(crate) fn deserialize_vec_set<'de, D, T>(deserializer: D) -> Result<HashSet<T>, D::Error>
where
  D: Deserializer<'de>,
  T: Deserialize<'de> + Eq + Hash,
{
  let vec_set = VecSet::<T>::deserialize(deserializer)?;
  Ok(vec_set.contents.into_iter().collect())
}

pub(crate) fn option_to_move<T: Serialize>(
  option: Option<T>,
  tag: TypeTag,
  ptb: &mut ProgrammableTransactionBuilder,
) -> Result<Argument, anyhow::Error> {
  let arg = if let Some(t) = option {
    let t = ptb.pure(t)?;
    ptb.programmable_move_call(
      MOVE_STDLIB_PACKAGE_ID,
      STD_OPTION_MODULE_NAME.into(),
      ident_str!("some").into(),
      vec![tag],
      vec![t],
    )
  } else {
    ptb.programmable_move_call(
      MOVE_STDLIB_PACKAGE_ID,
      STD_OPTION_MODULE_NAME.into(),
      ident_str!("none").into(),
      vec![tag],
      vec![],
    )
  };

  Ok(arg)
}

/// Create a VecSet from a vector of values
pub(crate) fn create_vec_set_from_move_values(
  values: Vec<Argument>,
  tag: TypeTag,
  ptb: &mut ProgrammableTransactionBuilder,
  package_id: ObjectID,
) -> Argument {
  let values = ptb.command(Command::MakeMoveVec(Some(tag.clone()), values));

  ptb.programmable_move_call(
    package_id,
    ident_str!("utils").into(),
    ident_str!("create_vec_set").into(),
    vec![tag],
    vec![values],
  )
}

/// Creates a new move string
pub(crate) fn new_move_string(
  value: String,
  ptb: &mut ProgrammableTransactionBuilder,
) -> anyhow::Result<Argument> {
  let v = ptb.pure(value.as_bytes())?;
  Ok(ptb.programmable_move_call(
    MOVE_STDLIB_PACKAGE_ID,
    STD_UTF8_MODULE_NAME.into(),
    ident_str!("utf8").into(),
    vec![],
    vec![v],
  ))
}

#[cfg(test)]
mod tests {
  use iota_sdk::types::collection_types::Entry;
  use serde_json::Value;

  use super::*;

  #[test]
  fn test_deserialize_vec_map_roundtrip() {
    let entry = Entry {
      key: 1,
      value: "value".to_string(),
    };
    let vec_map = VecMap {
      contents: vec![entry],
    };

    let json = serde_json::to_value(&vec_map).unwrap();

    // Use the custom deserializer
    let deserialized: HashMap<i32, String> = serde_json::from_value(json)
      .and_then(|value: Value| deserialize_vec_map(value))
      .unwrap();

    let mut expected = HashMap::new();
    expected.insert(1, "value".to_string());

    assert_eq!(deserialized, expected);
  }

  #[test]
  fn test_deserialize_vec_set() {
    let vec_set = VecSet {
      contents: vec!["1".to_string(), "2".to_string()],
    };

    let json = serde_json::to_value(&vec_set).unwrap();

    // Use the custom deserializer
    let deserialized: HashSet<String> = serde_json::from_value(json)
      .and_then(|value: Value| deserialize_vec_set(value))
      .unwrap();

    let mut expected = HashSet::new();
    expected.insert("1".to_string());
    expected.insert("2".to_string());

    assert_eq!(deserialized, expected);
  }
}
