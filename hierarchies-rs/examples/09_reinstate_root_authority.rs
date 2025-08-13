// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use hierarchies::core::types::Federation;
use hierarchies_examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;
use product_common::core_client::CoreClientReadOnly;

/// Demonstrate how to reinstate a previously revoked root authority in a federation.
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

    let federation_id = *federation.output.id.object_id();
    println!("Federation ID: {federation_id:#?}");

    // Add a second root authority first
    let second_root_authority = ObjectID::random();
    println!("Adding second root authority: {second_root_authority:#?}");

    hierarchies_client
        .add_root_authority(federation_id, second_root_authority)
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to add second root authority")?;

    // Check if the second root authority was added and is active
    let is_root_authority = hierarchies_client
        .is_root_authority(federation_id, second_root_authority)
        .await?;
    println!("Is second authority a root authority: {is_root_authority}");

    // Get the federation to see all root authorities
    let federation: Federation = hierarchies_client.get_object_by_id(federation_id).await?;
    println!(
        "Total root authorities before revocation: {}",
        federation.root_authorities.len()
    );

    // Now revoke the second root authority
    println!("Revoking second root authority: {second_root_authority:#?}");
    hierarchies_client
        .revoke_root_authority(federation_id, second_root_authority)
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to revoke root authority")?;

    println!("✅ Root authority revoked successfully!");

    // Verify the root authority was revoked
    let is_still_root_authority = hierarchies_client
        .is_root_authority(federation_id, second_root_authority)
        .await;

    match is_still_root_authority {
        Ok(false) => println!("✅ Root authority is no longer active"),
        Ok(true) => println!("❌ Root authority is still active (unexpected)"),
        Err(_) => println!("✅ Root authority check failed as expected (revoked authority)"),
    }

    // Get the updated federation after revocation
    let revoked_federation: Federation = hierarchies_client.get_object_by_id(federation_id).await?;
    println!(
        "Total root authorities after revocation: {}",
        revoked_federation.root_authorities.len()
    );
    println!(
        "Revoked root authorities count: {}",
        revoked_federation.revoked_root_authorities.len()
    );

    // Now reinstate the revoked root authority
    println!("\n🔄 Reinstating second root authority: {second_root_authority:#?}");
    hierarchies_client
        .reinstate_root_authority(federation_id, second_root_authority)
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to reinstate root authority")?;

    println!("✅ Root authority reinstated successfully!");

    // Verify the root authority was reinstated
    let is_reinstated_root_authority = hierarchies_client
        .is_root_authority(federation_id, second_root_authority)
        .await?;
    println!("Is second authority a root authority after reinstatement: {is_reinstated_root_authority}");

    // Get the final federation state
    let final_federation: Federation = hierarchies_client.get_object_by_id(federation_id).await?;
    println!("\nFinal federation state:");
    println!(
        "Total root authorities after reinstatement: {}",
        final_federation.root_authorities.len()
    );
    println!(
        "Revoked root authorities count: {}",
        final_federation.revoked_root_authorities.len()
    );

    // Print all active root authorities
    println!("Active root authorities:");
    final_federation
        .root_authorities
        .iter()
        .for_each(|ra| println!("  - Root Authority: {}", ra.account_id));

    // Print revoked root authorities (should be empty now)
    println!("Revoked root authorities:");
    final_federation
        .revoked_root_authorities
        .iter()
        .for_each(|ra| println!("  - Revoked Authority: {}", ra));

    Ok(())
}
