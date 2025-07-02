use std::collections::HashSet;

use anyhow::Context;
use examples::get_funded_client;
use ith::core::types::{Federation, StatementName, StatementValue};
use product_common::core_client::CoreClientReadOnly;

/// Demonstrate how to add a Statement to a federation.
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
    let another_value = StatementValue::from("World");
    let allowed_values = HashSet::from([value, another_value]);

    // Add the Statement to the federation
    {
        ith_client
            .add_statement(federation_id, statement_name.clone(), allowed_values, false)
            .build_and_execute(&ith_client)
            .await
            .context("Failed to add Statement")?;
    }

    // Get the updated federation and print it
    let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

    // Check if the Statement was added
    let statements = federation.governance.statements.contains_property(&statement_name);

    assert!(statements);

    if let Some(statement) = federation.governance.statements.data.get(&statement_name) {
        println!("Trusted Property: {:#?}", statement)
    }

    // Remove the Statement from the federation
    {
        ith_client
            .remove_statement(federation_id, statement_name.clone())
            .build_and_execute(&ith_client)
            .await
            .context("Failed to remove Statement")?;
    }

    // Get the updated federation and print it
    let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

    // Check if the Statement was removed
    let statements = federation.governance.statements.contains_property(&statement_name);

    assert!(!statements);

    Ok(())
}
