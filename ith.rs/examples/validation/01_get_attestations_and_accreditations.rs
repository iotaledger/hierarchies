use std::collections::HashSet;

use anyhow::Context;
use examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;
use ith::core::types::{Statement, StatementName, StatementValue, Timespan};
use product_common::core_client::CoreClient;

/// Demonstrates how to use the offchain API to check if a user has a permission to attest and accredit.
///
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.md

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ith_client = get_funded_client().await?;

    let federation = ith_client
        .create_new_federation()
        .build_and_execute(&ith_client)
        .await?;
    let federation_id = federation.output.id.object_id();

    let user_id = ith_client.sender_address().into();

    let permissions = ith_client.get_accreditations_to_attest(*federation_id, user_id).await?;

    println!("Accreditations: {:#?}", permissions);

    //   Add Statement
    let statement_name = StatementName::from("Example LTD");
    let value = StatementValue::from("Hello");
    let allowed_values = HashSet::from_iter([value]);

    ith_client
        .add_statement(*federation_id, statement_name.clone(), allowed_values.clone(), false)
        .build_and_execute(&ith_client)
        .await
        .context("Failed to add Statement")?;

    // Add new receiver
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
    {
        ith_client
            .create_accreditation_to_attest(*federation_id, receiver, vec![statements.clone()])
            .build_and_execute(&ith_client)
            .await
            .context("Failed to issue permission to attest")?;
    }

    // Check if the permission was issued
    let permissions = ith_client
        .get_accreditations_to_attest(*federation_id, receiver)
        .await
        .context("Failed to find permission to attest")?;

    assert!(permissions.statements.len() == 1);

    println!("Accreditations: {:#?}", permissions);

    // Issue Accredit permission
    {
        ith_client
            .create_accreditation_to_accredit(*federation_id, receiver, vec![statements])
            .build_and_execute(&ith_client)
            .await
            .context("Failed to issue permission to accredit")?;
    }

    // Check if the permission was issued
    let permissions = ith_client
        .get_accreditations_to_accredit(*federation_id, receiver)
        .await
        .context("Failed to find permission to accredit")?;

    assert!(permissions.statements.len() == 1);

    Ok(())
}
