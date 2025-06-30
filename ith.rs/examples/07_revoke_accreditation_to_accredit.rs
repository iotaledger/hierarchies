use std::collections::HashSet;

use anyhow::Context;
use examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;
use ith::core::types::{Federation, Statement, StatementName, StatementValue, Timespan};
use product_common::core_client::{CoreClient, CoreClientReadOnly};
/// Demonstrate how to issue a permission to accredit to a Statement.
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
    let ith_client = get_funded_client().await?;

    // Create new federation
    let federation = ith_client
        .create_new_federation()
        .build_and_execute(&ith_client)
        .await?;

    // Federation ID
    let federation_id = *federation.output.id.object_id();

    // Trusted property name
    let statement_name = StatementName::from("Example LTD");

    // Trusted property value
    let value = StatementValue::from("Hello");

    let allowed_values = HashSet::from_iter([value]);

    println!("Adding Statement");

    // Add the Statement to the federation
    ith_client
        .add_statement(federation_id, statement_name.clone(), allowed_values.clone(), false)
        .build_and_execute(&ith_client)
        .await
        .context("Failed to add Statement")?;

    println!("Added Statement");

    // A receiver is an account that will receive the accreditation
    let receiver = ObjectID::random();

    // Statements
    let statements = Statement {
        statement_name,
        allowed_values,
        expression: None,
        allow_any: false,
        timespan: Timespan::default(),
    };

    // Let us issue a permission to accredit to the Statement
    ith_client
        .create_accreditation_to_attest(federation_id, receiver, vec![statements.clone()])
        .build_and_execute(&ith_client)
        .await
        .context("Failed to issue permission to attest")?;

    // Issue permission to the original account
    ith_client
        .create_accreditation_to_attest(federation_id, ith_client.sender_address().into(), vec![statements])
        .build_and_execute(&ith_client)
        .await
        .context("Failed to issue permission to attest")?;

    println!("Issued permission to attest");

    // Check if the permission was issued
    let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

    // Check if the receiver has the permission to accredit
    let can_accredit = federation.governance.accreditations_to_accredit.contains_key(&receiver);

    assert!(can_accredit);

    // Revoke the permission
    let permissions = ith_client
        .get_accreditations_to_accredit(federation_id, receiver)
        .await
        .context("Failed to find permission to accredit")?;

    let permission_id = permissions.statements[0].id.object_id();

    ith_client
        .revoke_accreditation_to_accredit(federation_id, receiver, *permission_id)
        .build_and_execute(&ith_client)
        .await
        .context("Failed to revoke permission to accredit")?;

    // Check if the permission was revoked
    let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

    println!("Federation: {:#?}", federation);

    // Check if the receiver has the permission to accredit
    let can_accredit = federation.governance.accreditations_to_accredit.get(&receiver).unwrap();

    assert!(can_accredit.statements.is_empty());
    Ok(())
}
