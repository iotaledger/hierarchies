// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use hierarchies::client::get_object_ref_by_id_with_bcs;
use hierarchies::core::types::Federation;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::value::PropertyValue;
use product_common::core_client::{CoreClient, CoreClientReadOnly};

use crate::client::{TestClient, get_funded_test_client};

/// Helper function to create a federation for testing purposes.
/// Returns the federation object and transaction response.
async fn create_test_federation() -> anyhow::Result<(Federation, TestClient)> {
    let client = get_funded_test_client().await?;

    let federation_result = client.create_new_federation().build_and_execute(&client).await?;

    let federation = federation_result.output;

    Ok((federation, client))
}

#[tokio::test]
async fn test_add_property() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation first
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Create a property name and allowed values
    let property_name = PropertyName::from("test.credential.type");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("verified".to_string()));
    allowed_values.insert(PropertyValue::Text("pending".to_string()));

    // Add the property to the federation
    let result = client
        .add_property(
            *federation_id.object_id(),
            property_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await;

    assert!(result.is_ok(), "Failed to add property: {:?}", result.err());

    // Verify the property was added by fetching the federation
    let federation: Federation = get_object_ref_by_id_with_bcs(&client, federation_id.object_id()).await?;
    let properties = &federation.governance.properties.data;

    assert!(
        properties.contains_key(&property_name),
        "Property not found in federation"
    );
    let added_property = properties.get(&property_name).unwrap();
    assert_eq!(added_property.allowed_values, allowed_values);

    Ok(())
}

#[tokio::test]
async fn test_revoke_property() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation and add a property
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let property_name = PropertyName::from("test.temporary.credential");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("active".to_string()));

    // Add the property
    client
        .add_property(*federation_id.object_id(), property_name.clone(), allowed_values, false)
        .build_and_execute(&client)
        .await?;
    let result = client
        .revoke_property(*federation_id.object_id(), property_name.clone(), None)
        .build_and_execute(&client)
        .await;

    assert!(result.is_ok(), "Failed to revoke property: {:?}", result.err());

    Ok(())
}

#[tokio::test]
async fn test_create_and_get_properties() -> anyhow::Result<()> {
    let (federation, client) = create_test_federation().await?;

    let property_name = PropertyName::new(vec!["test_property"]);

    let property_values = HashSet::from_iter([PropertyValue::Text("test_value".to_string())]);

    client
        .add_property(
            *federation.id.object_id(),
            property_name.clone(),
            property_values,
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Get statements
    let properties = client.get_properties(*federation.id.object_id()).await?;

    assert_eq!(properties.len(), 1);

    let property = properties.first().unwrap();
    assert_eq!(property.clone(), property_name);

    Ok(())
}

#[tokio::test]
async fn test_create_and_validate_property() -> anyhow::Result<()> {
    let (federation, client) = create_test_federation().await?;

    let property_name = PropertyName::new(vec!["test_property"]);

    let property_value = PropertyValue::Text("test_value".to_string());

    let property_values = HashSet::from_iter([property_value.clone()]);
    client
        .add_property(
            *federation.id.object_id(),
            property_name.clone(),
            property_values,
            false,
        )
        .build_and_execute(&client)
        .await?;

    client
        .validate_property(
            *federation.id.object_id(),
            client.sender_address().into(),
            property_name.clone(),
            property_value,
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_add_property_with_allow_any() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Create a property that allows any value
    let property_name = PropertyName::from("test.open.field");
    let allowed_values = HashSet::new(); // Empty set when allow_any is true

    let result = client
        .add_property(*federation_id.object_id(), property_name.clone(), allowed_values, true)
        .build_and_execute(&client)
        .await;

    assert!(result.is_ok(), "Failed to add allow-any property: {:?}", result.err());

    // Verify the property was added
    let federation: Federation = client.get_object_by_id(*federation_id.object_id()).await?;
    let properties = &federation.governance.properties.data;

    assert!(properties.contains_key(&property_name));
    let added_property = properties.get(&property_name).unwrap();
    assert!(added_property.allow_any, "Property should allow any value");

    Ok(())
}

#[tokio::test]
async fn test_add_property_with_empty_allowed_values_and_allow_any_false_fails() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Try to add a property with empty allowed values and allow_any = false
    let property_name = PropertyName::from("test.invalid.property");
    let allowed_values = HashSet::new(); // Empty set

    let result = client
        .add_property(*federation_id.object_id(), property_name, allowed_values, false)
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_err(),
        "Should fail with empty allowed values and allow_any=false"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("10"),
        "Expected error code 10 for EEmptyAllowedValuesWithoutAllowAny, got: {error_msg}"
    );

    Ok(())
}

#[tokio::test]
async fn test_add_property_with_empty_allowed_values_and_allow_any_true_succeeds() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Add a property with empty allowed values and allow_any = true (should succeed)
    let property_name = PropertyName::from("test.any.value.property");
    let allowed_values = HashSet::new(); // Empty set

    let result = client
        .add_property(*federation_id.object_id(), property_name.clone(), allowed_values, true)
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Should succeed with empty allowed values and allow_any=true"
    );

    // Verify the property was added
    assert!(
        client
            .is_property_in_federation(*federation_id.object_id(), property_name)
            .await?
    );

    Ok(())
}

#[tokio::test]
async fn test_add_property_with_allowed_values_and_allow_any_false_succeeds() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Add a property with specific allowed values and allow_any = false (should succeed)
    let property_name = PropertyName::from("test.restricted.property");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Number(1));
    allowed_values.insert(PropertyValue::Number(2));

    let result = client
        .add_property(*federation_id.object_id(), property_name.clone(), allowed_values, false)
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Should succeed with non-empty allowed values and allow_any=false"
    );

    // Verify the property was added
    assert!(
        client
            .is_property_in_federation(*federation_id.object_id(), property_name)
            .await?
    );

    Ok(())
}
