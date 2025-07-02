use std::collections::HashSet;

use anyhow::Context;
use examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;
use ith::core::types::statements::name::StatementName;
use ith::core::types::statements::value::StatementValue;
use ith::core::types::statements::Statement;

/// Getting started
///
/// When the ITH package is published it creates a new federation,
/// adds Statements, creates an attestation, validates them,
/// Before running the example:
///  - ensure you have the IOTA CLI installed and configured for the the selected network
///  - ITH_ITH_PKG_ID env is set to the ITH package ID
///  - API_URL env is set to the IOTA node URL
///
/// Please note that we use an unsecured private key provider [`TestMemSigner`],
/// which should NOT be used in production.
///
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = get_funded_client().await?;

    // Create a Statement with allowed values
    let statement_name = StatementName::new(["university", "a", "score", "department"]);
    let value_biology = StatementValue::Text("biology".to_owned());
    let value_physics = StatementValue::Text("physics".to_owned());

    // Allowed values for the statement in whole federation
    let allowed_values_statements = HashSet::from([value_biology.clone(), value_physics.clone()]);

    // Create new federation
    println!("Creating a new federation");
    let federation = client.create_new_federation().build_and_execute(&client).await?;
    println!("Federation created");
    let federation_id = *federation.output.id.object_id();

    println!("Adding trusted statements");
    // Add the Statement to the federation. The federation owner can add Statements by default.
    client
        .add_statement(federation_id, statement_name.clone(), allowed_values_statements, false)
        .build_and_execute(&client)
        .await
        .context("Failed to add a Statement")?;
    println!("✅ Added Statement");

    // Lets delegate the trust to another account and create an accreditation withe the Statement
    // The receiver account will be able to attest to the Statement `university.a.score.department`
    // and value `physics` on behalf of the Federation

    // An attester is an account that will receive the accreditation to attest
    let attester = ObjectID::random();
    // Allowed values for the attestation
    let allowed_values_attestation = [value_physics.clone()];

    // Statements
    let statements = Statement::new(statement_name.clone()).with_allowed_values(allowed_values_attestation);

    // Create an accreditation to attest to the Statement
    client
        .create_accreditation_to_attest(federation_id, attester, [statements])
        .build_and_execute(&client)
        .await
        .context("Failed creating attestation")?;
    println!(
        "✅ Accreditation to attest has been created for the user {attestation_receiver}"
    );

    // Let's validate the Statements. Validation is a process of checking if the accreditation
    // receiver is accredited to attest to the Statement with the given Statement Value
    client
        .validate_statements(
            federation_id,
            attester,
            [(statement_name.clone(), value_physics.clone())],
        )
        .await
        .context("Failed to validate Statements")?;
    println!("✅ Validated Statements");

    // TODO replace with revoke_accreditation_to_attest
    client
        .remove_statement(federation_id, statement_name.clone())
        .build_and_execute(&client)
        .await
        .context("Failed to revoke attestation")?;

    println!("✅ Revoked attestation");

    // Validate Statements again - it should returned an error
    let expected_error = client
        .validate_statements(
            federation_id,
            attester,
            [(statement_name.clone(), value_physics.clone())],
        )
        .await;
    assert!(expected_error.is_err());
    println!(
        "✅ Expected error on validation after revocation for '{value_physics:?}'"
    );

    println!("🎉 Done");
    Ok(())
}
