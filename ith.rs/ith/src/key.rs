use iota_sdk::types::base_types::IotaAddress;
use secret_storage::SignatureScheme;

pub struct IotaKeySignature {
  pub public_key: Vec<u8>,
  pub signature: Vec<u8>,
}

impl SignatureScheme for IotaKeySignature {
  type Input = Vec<u8>;
  type PublicKey = Vec<u8>;

  type Signature = Vec<u8>;
}

#[derive(Clone)]
pub struct SigningInfo {
  pub sender_public_key: Vec<u8>,
  pub sender_address: IotaAddress,
}
