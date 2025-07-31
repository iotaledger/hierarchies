// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_interaction::types::base_types::ObjectID;
use product_common::core_client::CoreClient;

use crate::client::get_funded_test_client;

#[tokio::test]
async fn test_add_root_authority_success() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation = client
        .create_new_federation()
        .build_and_execute(&client)
        .await
        .unwrap()
        .output
        .id;

    let bob_id = ObjectID::random();

    // Add Bob as root authority
    client
        .add_root_authority(*federation.object_id(), bob_id)
        .build_and_execute(&client)
        .await?;

    // Verify Bob is a root authority
    assert!(client.is_root_authority(*federation.object_id(), bob_id).await?);

    Ok(())
}

#[tokio::test]
async fn test_revoke_root_authority_success() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation = client
        .create_new_federation()
        .build_and_execute(&client)
        .await
        .unwrap()
        .output
        .id;

    let bob_id = ObjectID::random();
    let charlie_id = ObjectID::random();

    // Add Bob as root authority
    client
        .add_root_authority(*federation.object_id(), bob_id)
        .build_and_execute(&client)
        .await?;

    // Add Charlie as root authority
    client
        .add_root_authority(*federation.object_id(), charlie_id)
        .build_and_execute(&client)
        .await?;

    // Verify all three are root authorities
    let alice_id = ObjectID::from_address(client.sender_address().into());
    assert!(client.is_root_authority(*federation.object_id(), alice_id).await?);
    assert!(client.is_root_authority(*federation.object_id(), bob_id).await?);
    assert!(client.is_root_authority(*federation.object_id(), charlie_id).await?);

    // Revoke Bob as root authority
    client
        .revoke_root_authority(*federation.object_id(), bob_id)
        .build_and_execute(&client)
        .await?;

    // Verify Bob is no longer a root authority
    assert!(client.is_root_authority(*federation.object_id(), alice_id).await?);
    assert!(!client.is_root_authority(*federation.object_id(), bob_id).await?);
    assert!(client.is_root_authority(*federation.object_id(), charlie_id).await?);

    Ok(())
}

#[tokio::test]
async fn test_revoke_root_authority_not_found() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation = client
        .create_new_federation()
        .build_and_execute(&client)
        .await
        .unwrap()
        .output
        .id;

    let non_existent_id = ObjectID::random();

    // Try to revoke a non-existent root authority
    let result = client
        .revoke_root_authority(*federation.object_id(), non_existent_id)
        .build_and_execute(&client)
        .await;

    assert!(result.is_err());
    // Check that the error message contains expected content
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("8"));

    Ok(())
}

#[tokio::test]
async fn test_cannot_revoke_last_root_authority() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation = client
        .create_new_federation()
        .build_and_execute(&client)
        .await
        .unwrap()
        .output
        .id;

    let alice_id = ObjectID::from_address(client.sender_address().into());

    // Try to revoke the only root authority (Alice)
    let result = client
        .revoke_root_authority(*federation.object_id(), alice_id)
        .build_and_execute(&client)
        .await;

    assert!(result.is_err());
    // Check that the error message contains expected content
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("9"));

    Ok(())
}

#[tokio::test]
async fn test_is_root_authority() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation = client
        .create_new_federation()
        .build_and_execute(&client)
        .await
        .unwrap()
        .output
        .id;

    let alice_id = ObjectID::from_address(client.sender_address().into());
    let bob_id = ObjectID::random();
    let charlie_id = ObjectID::random();

    // Initially only Alice is a root authority
    assert!(client.is_root_authority(*federation.object_id(), alice_id).await?);
    assert!(!client.is_root_authority(*federation.object_id(), bob_id).await?);
    assert!(!client.is_root_authority(*federation.object_id(), charlie_id).await?);

    // Add Bob as root authority
    client
        .add_root_authority(*federation.object_id(), bob_id)
        .build_and_execute(&client)
        .await?;

    // Now both Alice and Bob are root authorities
    assert!(client.is_root_authority(*federation.object_id(), alice_id).await?);
    assert!(client.is_root_authority(*federation.object_id(), bob_id).await?);
    assert!(!client.is_root_authority(*federation.object_id(), charlie_id).await?);

    Ok(())
}
