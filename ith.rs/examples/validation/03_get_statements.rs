// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use anyhow::Context;
use ith::core::types::statements::name::StatementName;
use ith::core::types::statements::value::StatementValue;
use trust_hierarchies_examples::get_funded_client;

/// Demonstrates how to use the offchain API to get federation properties.
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

    //   Add Statement
    {
        let statement_name = StatementName::from("Example LTD");
        let value = StatementValue::Text("Hello".to_owned());
        let allowed_values = HashSet::from_iter([value.clone()]);

        // Add the Statement to the federation
        ith_client
            .add_statement(*federation_id, statement_name.clone(), allowed_values.clone(), false)
            .build_and_execute(&ith_client)
            .await
            .context("Failed to add Statement")?;
    }

    // Add second Statement
    {
        let statement_name = StatementName::new(["Example LTD 2", "Example LTD 3"]);
        let value = StatementValue::Text("Hello 2".to_owned());
        let allowed_values = HashSet::from_iter([value.clone()]);

        // Add the Statement to the federation
        ith_client
            .add_statement(*federation_id, statement_name.clone(), allowed_values.clone(), false)
            .build_and_execute(&ith_client)
            .await
            .context("Failed to add Statement")?;
    }

    let federation_properties = ith_client.get_statements(*federation_id).await?;

    assert!(federation_properties.len() == 2);

    println!("Federation properties: {federation_properties:#?}");

    Ok(())
}
