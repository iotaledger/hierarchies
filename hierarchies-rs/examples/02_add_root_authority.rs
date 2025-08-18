// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use hierarchies::core::types::Federation;
use hierarchies_examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;
use product_common::core_client::CoreClientReadOnly;

/// Demonstrate how to add a root authority to a federation.
///
/// In this example we connect to a locally running private network, but it can
/// be adapted to run on any IOTA node by setting the network and faucet
/// endpoints.
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.mdx

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get the client instance
    let hierarchies_client = get_funded_client().await?;

    // Create new federation
    let federation = hierarchies_client
        .create_new_federation()
        .build_and_execute(&hierarchies_client)
        .await?;

    // Create a new root authority object ID
    let new_root_authority = ObjectID::random();
    println!("New Root Authority: {new_root_authority:#?}");

    // Federation ID
    let federation_id = *federation.output.id.object_id();

    // Add the root authority to the federation
    hierarchies_client
        .add_root_authority(federation_id, new_root_authority)
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to add Root Authority")?;

    // Get the updated federation and print it
    let federation: Federation = hierarchies_client.get_object_by_id(federation_id).await?;
    println!("New Federation created.");

    // Check if the root authority was added
    let root_authorities = federation.root_authorities;

    // Print the root authorities
    root_authorities
        .iter()
        .filter(|ra| ra.account_id == new_root_authority)
        .for_each(|ra| println!("Root Authority: {ra:#?}"));

    Ok(())
}
