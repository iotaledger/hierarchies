use std::collections::HashMap;
use std::hash::Hash;

use fastcrypto::ed25519::Ed25519PublicKey;
use fastcrypto::traits::ToFromBytes;
use iota_sdk::types::base_types::IotaAddress;
use iota_sdk::types::collection_types::VecMap;
use iota_sdk::types::id::ID;
use iota_sdk::types::id::UID;
use serde::Deserialize;
use serde::Serialize;

pub fn convert_to_address(sender_public_key: &[u8]) -> anyhow::Result<IotaAddress> {
  let public_key = Ed25519PublicKey::from_bytes(sender_public_key)
    .map_err(|err| anyhow::anyhow!(format!("could not parse public key to Ed25519 public key; {err}")))?;

  Ok(IotaAddress::from(&public_key))
}

pub trait IntoHash {
  fn as_hash(&self) -> impl Hash;
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
#[repr(transparent)]
#[serde(transparent)]
pub(crate) struct Hashable<T>(pub T);

impl<T: IntoHash> Hash for Hashable<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.0.as_hash().hash(state);
  }
}

impl IntoHash for ID {
  fn as_hash(&self) -> impl Hash {
    &self.bytes
  }
}

impl IntoHash for UID {
  fn as_hash(&self) -> impl Hash {
    self.id.as_hash()
  }
}

pub trait IntoCollectionHash<K, V> {
  fn to_hashmap(self) -> HashMap<K, V>;
}

impl<K: Eq + Hash, V> IntoCollectionHash<K, V> for VecMap<K, V> {
  fn to_hashmap(self) -> HashMap<K, V> {
    self
      .contents
      .into_iter()
      .map(|entry| (entry.key, entry.value))
      .collect()
  }
}
