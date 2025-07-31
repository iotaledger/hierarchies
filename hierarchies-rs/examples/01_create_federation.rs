// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use hierarchies_examples::get_funded_client;

/// Demonstrates how to create a Federation and publish it on chain.
///
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.mdx

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hierarchies_client = get_funded_client()
        .await
        .map_err(|err| anyhow::anyhow!(format!("failed to create Hierarchies client; {}", err)))?;

    println!("Creating new federation");

    let federation = hierarchies_client
        .create_new_federation()
        .build_and_execute(&hierarchies_client)
        .await?
        .output;

    println!("Federation created: {federation:#?}");

    Ok(())
}
