
use iota_sdk::types::base_types::IotaAddress;

use secret_storage::prelude::KeySignatureTypes;

pub struct IotaKeySignature {
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

impl KeySignatureTypes for IotaKeySignature {
    type PublicKey = Vec<u8>;

    type Signature = Vec<u8>;
}

/// Mirrored types from identity_storage::KeyId
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct KeyId(String);

impl KeyId {
    /// Creates a new key identifier from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns string representation of the key id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<KeyId> for String {
    fn from(value: KeyId) -> Self {
        value.0
    }
}

#[derive(Clone)]
pub struct SigningInfo {
    pub sender_public_key: Vec<u8>,
    pub sender_address: IotaAddress,
}
