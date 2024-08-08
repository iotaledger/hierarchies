use fastcrypto::ed25519::Ed25519PublicKey;
use fastcrypto::traits::ToFromBytes;
use iota_sdk::types::base_types::IotaAddress;

pub fn convert_to_address(sender_public_key: &[u8]) -> anyhow::Result<IotaAddress> {
    let public_key = Ed25519PublicKey::from_bytes(sender_public_key).map_err(|err| {
        anyhow::anyhow!(format!(
            "could not parse public key to Ed25519 public key; {err}"
        ))
    })?;

    Ok(IotaAddress::from(&public_key))
}
