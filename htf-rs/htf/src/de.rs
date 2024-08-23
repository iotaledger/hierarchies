use iota_sdk::types::collection_types::VecMap;
use iota_sdk::types::collection_types::VecSet;
use serde::Deserialize;
use serde::Deserializer;

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

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
  use super::*;
  use iota_sdk::types::collection_types::Entry;
  use serde_json::Value;

  #[test]
  fn test_deserialize_vec_map_roundtrip() {
    let entry = Entry {
      key: 1,
      value: "value".to_string(),
    };
    let vec_map = VecMap { contents: vec![entry] };

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
