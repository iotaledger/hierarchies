use anyhow::Context;
use examples::{get_client, urls};
use iota_sdk::types::base_types::ObjectID;
use ith::types::TrustedPropertyConstraint;
use ith::types::{TrustedPropertyName, TrustedPropertyValue};

/// Getting started
///
/// This example automatically installs ITH package in the testnet network.
/// When the ITH package is published it creates a new federation,
/// adds trusted properties, creates an attestation, validates them,
/// revokes the attestation, then validates them again.
///
/// Before running the example:
///  - ensure you have the IOTA CLI installed and configured for the testnet network
///
/// Please note that we use an unsecured private key provider [`TestMemSigner`],
/// which should NOT be used in production.
///
/// *The ID of newly deployed package is cached in the file `ith.rs/target/ith_pkg_id.txt`.
/// *If you want to re-deploy the package, you need to remove the file.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let client = get_client(urls::testnet::node(), urls::testnet::faucet()).await?;

  // Create a trusted property with allowed values
  let property_name = TrustedPropertyName::new(["university", "a", "score", "department"]);
  let value_biology = TrustedPropertyValue::from("biology");
  let value_physics = TrustedPropertyValue::from("physics");

  // Allowed values for the property in whole federation
  let allowed_values_property = [value_biology.clone(), value_physics.clone()];

  // Create new federation
  println!("creating a new federation");
  let federation = client.new_federation(None).await?;
  println!("federation crated");
  let federation_id = *federation.id.object_id();

  println!("adding trust properties");
  // Add the trusted property to the federation. The federation owner can add trusted properties.
  client
    .add_trusted_property(
      federation_id,
      property_name.clone(),
      allowed_values_property,
      false,
      None,
    )
    .await
    .context("Failed to add trusted property")?;
  println!("✅ Added trusted property");

  // Lets delegate the trust to another account and create an attestation for the property
  // The receiver account will be able to attest to the property `university.a.score.department`
  // and value `physics` on behalf of the federation

  // A receiver is an account that will receive the attestation
  let attestation_receiver = ObjectID::random();
  // Allowed values for the attestation
  let allowed_values_attestation = [value_physics.clone()];

  // Property constraints
  let constraints = TrustedPropertyConstraint::new(property_name.clone())
    .with_allowed_values(allowed_values_attestation);

  // Create an attestation
  client
    .create_attestation(federation_id, attestation_receiver, [constraints], None)
    .await
    .context("Failed creating attestation")?;
  println!(
    "✅ Attestation has been created for user {}",
    attestation_receiver
  );

  // Let's validate the trusted properties. Validation is a process of checking if the attestation
  // receiver has the right to attest to the property with the given value.
  //
  // The validation can be done on-chain or off-chain.
  // The off-chain validation is a zero cost operation, while the on-chain validation is a low cost operation.

  // On-chain validation (low cost):
  client
    .onchain(federation_id)
    .validate_trusted_properties(
      attestation_receiver,
      [(property_name.clone(), value_physics.clone())],
    )
    .await
    .context("Failed to validate trusted properties")?;
  println!("✅ Validated trusted properties - ON-CHAIN");

  // Off-chain validation (zero cost):
  client
    .offchain(federation_id)
    .await?
    .validate_trusted_properties(
      attestation_receiver,
      [(property_name.clone(), value_physics.clone())],
    )
    .context("Failed to validate trusted properties")?;
  println!("✅ Validated trusted properties - OFF-CHAIN");

  // Revoke just created attestation
  let attestations = client
    .onchain(federation_id)
    .get_attestations(attestation_receiver)
    .await?;
  let attestation_id = attestations.permissions[0].id.object_id();

  client
    .revoke_attestation(federation_id, attestation_receiver, *attestation_id, None)
    .await
    .context("Failed to revoke attestation")?;
  println!("✅ Revoked attestation");

  // Validate trusted properties again - it should returned an error
  let expected_error = client
    .onchain(federation_id)
    .validate_trusted_properties(
      attestation_receiver,
      [(property_name.clone(), value_physics.clone())],
    )
    .await;
  assert!(expected_error.is_err());
  println!(
    "✅ Expected error on validation after revocation for '{:?}'",
    value_physics
  );

  println!("🎉 Done");
  Ok(())
}
