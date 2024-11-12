use anyhow::Context;
use examples::TestMemSigner;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::IotaClientBuilder;
use ith::client::ITHClientReadOnly;
use ith::types::TrustedPropertyConstraint;
use ith::types::{TrustedPropertyName, TrustedPropertyValue};

/// Getting started
///
/// In this example we connect to the already deployed ITH package on the IOTA test network.
/// We create a new federation, add a trusted property to it, create an attestation,
/// validate the trusted properties, revoke the attestation and validate the trusted properties again.
///
/// Before you run the example:
///  - make sure to set the `ITH_PKG_ID` environment variable with the package id of the deployed ITH package.
///  - the signer's account that will be used for the example should have some tokens to perform the operations.
/// Please be aware that we use UNSECURE private key provider - [`TestMemSigner`]. It should NOT be used in production.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Get the package id of the deployed ITH package
  let package_id = std::env::var("ITH_PKG_ID").expect("ITH_PKG_ID is not set");

  // A test signer is used for demonstration purposes only. Please make sure the signer's account has enough tokens.
  let signer = TestMemSigner::new();

  // We need an IOTA network client to interact as a dependency for the ITH client.
  let iota_client = IotaClientBuilder::default().build_testnet().await?;

  // Create a read-only client for the ITH. The read-only client is used for
  // read-only operations and does't require a signer with a private key. It is used for off-chain operations.
  let read_only_client = ITHClientReadOnly::new(iota_client, package_id.parse()?);

  // Create a new ITH client with the read-only client and the signer.
  let client = ith::client::ITHClient::new(read_only_client, signer).await?;

  // Create a trusted property with allowed values
  let property_name = TrustedPropertyName::new(["university", "a", "score", "department"]);
  let value_biology = TrustedPropertyValue::from("biology");
  let value_physics = TrustedPropertyValue::from("physics");

  // Allowed values for the property in whole federation
  let allowed_values_property = [value_biology.clone(), value_physics.clone()];

  // Create new federation
  let federation = client.new_federation(None).await?;
  let federation_id = *federation.id.object_id();

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
