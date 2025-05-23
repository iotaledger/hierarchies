use anyhow::Context;
use examples::{get_client, urls};
use iota_sdk::types::base_types::ObjectID;
use ith::types::Statement;
use ith::types::{StatementName, StatementValue};

/// Getting started
///
/// This example automatically installs ITH package in the testnet network.
/// When the ITH package is published it creates a new federation,
/// adds Statements, creates an attestation, validates them,
/// revokes the accreditation to attest, then validates them again.
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

  // Create a Statement with allowed values
  let statement_name = StatementName::new(["university", "a", "score", "department"]);
  let value_biology = StatementValue::from("biology");
  let value_physics = StatementValue::from("physics");

  // Allowed values for the property in whole federation
  let allowed_values_property = [value_biology.clone(), value_physics.clone()];

  // Create new federation
  println!("creating a new federation");
  let federation = client.new_federation(None).await?;
  println!("federation crated");
  let federation_id = *federation.id.object_id();

  println!("adding trust properties");
  // Add the Statement to the federation. The federation owner can add Statements by default.
  client
    .add_statement(
      federation_id,
      statement_name.clone(),
      allowed_values_property,
      false,
      None,
    )
    .await
    .context("Failed to add Statement")?;
  println!("✅ Added Statement");

  // Lets delegate the trust to another account and create an accreditation withe the Statement
  // The receiver account will be able to attest to the Statement `university.a.score.department`
  // and value `physics` on behalf of the Federation

  // A receiver is an account that will receive the accreditation
  let attestation_receiver = ObjectID::random();
  // Allowed values for the attestation
  let allowed_values_attestation = [value_physics.clone()];

  // Statements
  let statements =
    Statement::new(statement_name.clone()).with_allowed_values(allowed_values_attestation);

  // Create an accreditation to attest to the Statement
  client
    .create_accreditation_to_attest(federation_id, attestation_receiver, [statements], None)
    .await
    .context("Failed creating attestation")?;
  println!(
    "✅ Accreditation to attest has been created for the user {}",
    attestation_receiver
  );

  // Let's validate the Statements. Validation is a process of checking if the accreditation
  // receiver is accredited to attest to the Statement with the given Statement Value
  //
  // The validation can be done on-chain or off-chain.
  // The off-chain validation is a zero cost operation, while the on-chain validation is a low cost operation.

  // On-chain validation (low cost):
  client
    .onchain(federation_id)
    .validate_statements(
      attestation_receiver,
      [(statement_name.clone(), value_physics.clone())],
    )
    .await
    .context("Failed to validate Statements")?;
  println!("✅ Validated Statements - ON-CHAIN");

  // Off-chain validation (zero cost):
  client
    .offchain(federation_id)
    .await?
    .validate_statements(
      attestation_receiver,
      [(statement_name.clone(), value_physics.clone())],
    )
    .context("Failed to validate Statements")?;
  println!("✅ Validated Statements - OFF-CHAIN");

  // Revoke just created accreditation to attest
  let attestations = client
    .onchain(federation_id)
    .get_accreditations_to_attest(attestation_receiver)
    .await?;
  let attestation_id = attestations.permissions[0].id.object_id();

  client
    .revoke_accreditation_to_attest(federation_id, attestation_receiver, *attestation_id, None)
    .await
    .context("Failed to revoke attestation")?;
  println!("✅ Revoked attestation");

  // Validate Statements again - it should returned an error
  let expected_error = client
    .onchain(federation_id)
    .validate_statements(
      attestation_receiver,
      [(statement_name.clone(), value_physics.clone())],
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
