use anyhow::Context;
use examples::get_client;
use ith::types::trusted_constraints::TrustedPropertyConstraint;
use ith::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use iota_sdk::types::base_types::ObjectID;

/// Demonstrate how to issue a permission to attest to a trusted property.
///
/// In this example we connect to a locally running private network, but it can
/// be adapted to run on any IOTA node by setting the network and faucet
/// endpoints.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Get the client instance
  let client = get_client().await?;

  let property_name = TrustedPropertyName::new(["vc", "type"]);
  let value_verifiable_credential = TrustedPropertyValue::from("VerifiableCredential");
  let value_credential_degree = TrustedPropertyValue::from("ExampleDegreeCredential");

  let allowed_values_property = [
    value_verifiable_credential.clone(),
    value_credential_degree.clone(),
  ];
  let allowed_values_attestation = [value_credential_degree.clone()];
  // A receiver is an account that will receive the attestation
  let attestation_receiver = ObjectID::random();

  // Create new federation
  //
  let federation = client.new_federation(None).await?;
  let federation_id = *federation.id.object_id();

  // Add the trusted property to the federation
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

  // Property constraints
  let constraints = TrustedPropertyConstraint::new(property_name.clone())
    .with_allowed_values(allowed_values_attestation);

  // Create an attestation
  client
    .create_attestation(federation_id, attestation_receiver, [constraints], None)
    .await
    .context("Failed creating attestation")?;
  println!("✅ Attestation has been created",);

  // Validate trusted properties
  // On-chain (low cost):
  client
    .onchain(federation_id)
    .validate_trusted_properties(
      attestation_receiver,
      [(property_name.clone(), value_credential_degree.clone())],
    )
    .await
    .context("Failed to validate trusted properties")?;
  println!("✅ Validated trusted properties - ON-CHAIN");

  // Validate trusted properties
  // Off-chain (zero cost):
  client
    .offchain(federation_id)
    .await?
    .validate_trusted_properties(
      attestation_receiver,
      [(property_name.clone(), value_credential_degree.clone())],
    )
    .context("Failed to validate trusted properties")?;
  println!("✅ Validated trusted properties - OFF-CHAIN");

  // Validate with wrong property value
  let expected_error = client
    .onchain(federation_id)
    .validate_trusted_properties(
      attestation_receiver,
      [(property_name.clone(), value_verifiable_credential.clone())],
    )
    .await;
  assert!(expected_error.is_err());
  println!(
    "✅ Expected error on validation for '{:?}'",
    value_verifiable_credential,
  );

  // Revoke the just created attestation
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
      [(property_name.clone(), value_credential_degree.clone())],
    )
    .await;
  assert!(expected_error.is_err());
  println!(
    "✅ Expected error on validation after revocation for '{:?}'",
    value_credential_degree
  );

  println!("🎉 Done");
  Ok(())
}
