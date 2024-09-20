use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use fastcrypto::ed25519::Ed25519PublicKey;
use fastcrypto::traits::ToFromBytes;
use iota_sdk::types::base_types::IotaAddress;
use iota_sdk::types::collection_types::{VecMap, VecSet};
use serde::{Deserialize, Deserializer};

pub fn convert_to_address(sender_public_key: &[u8]) -> anyhow::Result<IotaAddress> {
  let public_key = Ed25519PublicKey::from_bytes(sender_public_key).map_err(|err| {
    anyhow::anyhow!(format!(
      "could not parse public key to Ed25519 public key; {err}"
    ))
  })?;

  Ok(IotaAddress::from(&public_key))
}

pub fn deserialize_vec_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
  D: Deserializer<'de>,
  K: Deserialize<'de> + Eq + Hash,
  V: Deserialize<'de>,
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

pub fn deserialize_vec_set<'de, D, T>(deserializer: D) -> Result<HashSet<T>, D::Error>
where
  D: Deserializer<'de>,
  T: Deserialize<'de> + Eq + Hash,
{
  let vec_set = VecSet::<T>::deserialize(deserializer)?;
  Ok(vec_set.contents.into_iter().collect())
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
