use std::collections::HashSet;

use anyhow::Context;
use examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;
use ith::core::types::statements::name::StatementName;
use ith::core::types::statements::value::StatementValue;
use ith::core::types::statements::Statement;
use ith::core::types::timespan::Timespan;
use ith::core::types::Federation;
use product_common::core_client::CoreClient;

/// Demonstrate how to issue a permission to attest to a Statement.
///
/// In this example we connect to a locally running private network, but it can
/// be adapted to run on any IOTA node by setting the network and faucet
/// endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.mdx

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get the client instance
    let client = get_funded_client().await?;

    // Create new federation
    let federation = client.create_new_federation().build_and_execute(&client).await?;

    // Federation ID
    let federation_id = *federation.output.id.object_id();

    // Trusted property name
    let statement_name = StatementName::from("Example LTD");

    // Trusted property value
    let value = StatementValue::Text("Hello".to_owned());

    let allowed_values = HashSet::from_iter([value]);

    println!("Adding Statement");

    // Add the Statement to the federation
    client
        .add_statement(federation_id, statement_name.clone(), allowed_values.clone(), false)
        .build_and_execute(&client)
        .await
        .context("Failed to add Statement")?;

    println!("Added Statement");

    // A receiver is an account that will receive the attestation
    let receiver = ObjectID::random();

    // Statements
    let statements = Statement {
        statement_name,
        allowed_values,
        condition: None,
        allow_any: false,
        timespan: Timespan::default(),
    };

    // Let us issue a permission to attest to the Statement
    client
        .create_accreditation_to_attest(federation_id, receiver, vec![statements.clone()])
        .build_and_execute(&client)
        .await
        .context("Failed to issue permission to attest")?;

    // Issue permission to the original account
    client
        .create_accreditation_to_attest(federation_id, client.sender_address().into(), vec![statements])
        .build_and_execute(&client)
        .await
        .context("Failed to issue permission to attest")?;

    println!("Issued permission to attest");

    // Check if the permission was issued
    let federation: Federation = client.get_federation_by_id(federation_id).await?;

    // Check if the receiver has the permission to attest
    let can_attest = federation.governance.accreditations_to_attest.contains_key(&receiver);

    assert!(can_attest);

    // Revoke the permission
    let statements = client
        .get_accreditations_to_attest(federation_id, receiver)
        .await
        .context("Failed to find permission to attest")?;

    let permission_id = statements.statements[0].id.object_id();

    client
        .revoke_accreditation_to_attest(federation_id, receiver, *permission_id)
        .build_and_execute(&client)
        .await
        .context("Failed to revoke permission to attest")?;

    // Check if the permission was revoked
    let federation: Federation = client.get_federation_by_id(federation_id).await?;

    println!("Federation: {federation:#?}");

    // Check if the receiver has the permission to attest
    let can_attest = federation.governance.accreditations_to_attest.get(&receiver).unwrap();

    assert!(can_attest.statements.is_empty());
    Ok(())
}
