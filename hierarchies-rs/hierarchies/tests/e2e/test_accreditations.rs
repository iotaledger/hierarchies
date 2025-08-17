// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use hierarchies::core::types::Federation;
use hierarchies::core::types::property::FederationProperty;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::property_value::PropertyValue;
use iota_interaction::types::base_types::ObjectID;

use crate::client::get_funded_test_client;

#[tokio::test]
async fn test_create_accreditation_to_attest() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation and add properties
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let property_name = PropertyName::from("certification.level");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("basic".to_string()));
    allowed_values.insert(PropertyValue::Text("advanced".to_string()));

    client
        .add_property(
            *federation_id.object_id(),
            property_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create properties for the accreditation
    let mut accreditation_values = HashSet::new();
    accreditation_values.insert(PropertyValue::Text("basic".to_string()));

    let property = FederationProperty::new(property_name).with_allowed_values(accreditation_values);

    // Create accreditation to attest for a test receiver
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_attest(*federation_id.object_id(), receiver_id, vec![property])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Failed to create accreditation to attest: {:?}",
        result.err()
    );

    // Verify the accreditation was created by checking the federation
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    assert!(!federation.governance.accreditations_to_attest.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_to_accredit() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation and add properties
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let property_name = PropertyName::from("accreditation.authority");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("issuer".to_string()));

    client
        .add_property(
            *federation_id.object_id(),
            property_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create property for the accreditation
    let property = FederationProperty::new(property_name).with_allowed_values(allowed_values);

    // Create accreditation to accredit for a test receiver
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_accredit(*federation_id.object_id(), receiver_id, vec![property])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Failed to create accreditation to accredit: {:?}",
        result.err()
    );

    // Verify the accreditation was created
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    assert!(!federation.governance.accreditations_to_accredit.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_multiple_statements_in_federation() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Add multiple different properties
    let properties_to_add = vec![
        (
            PropertyName::from("identity.verification.level"),
            vec![
                PropertyValue::Text("basic".to_string()),
                PropertyValue::Text("enhanced".to_string()),
            ],
            false,
        ),
        (
            PropertyName::from("identity.age.minimum"),
            vec![PropertyValue::Number(18), PropertyValue::Number(21)],
            false,
        ),
        (
            PropertyName::from("identity.custom.field"),
            vec![], // Empty for allow_any = true
            true,
        ),
    ];

    // Add all properties
    for (name, values, allow_any) in properties_to_add.iter() {
        let allowed_values: HashSet<PropertyValue> = values.iter().cloned().collect();
        client
            .add_property(*federation_id.object_id(), name.clone(), allowed_values, *allow_any)
            .build_and_execute(&client)
            .await?;
    }

    // Verify all properties were added
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    let properties = &federation.governance.properties.data;

    assert_eq!(properties.len(), properties_to_add.len());

    for (name, values, allow_any) in properties_to_add {
        assert!(properties.contains_key(&name), "Property {name:?} not found");
        let property = properties.get(&name).unwrap();
        assert_eq!(property.allow_any, allow_any, "Allow any mismatch for {name:?}");

        if !allow_any {
            let expected_values: HashSet<PropertyValue> = values.iter().cloned().collect();
            assert_eq!(property.allowed_values, expected_values, "Values mismatch for {name:?}");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_revoke_accreditation_to_attest() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create federation and add property
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let property_name = PropertyName::from("test.revokable");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("value".to_string()));

    client
        .add_property(
            *federation_id.object_id(),
            property_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create accreditation to attest
    let receiver_id = ObjectID::random();
    let statement = FederationProperty::new(property_name).with_allowed_values(allowed_values);

    client
        .create_accreditation_to_attest(*federation_id.object_id(), receiver_id, vec![statement])
        .build_and_execute(&client)
        .await?;

    // Get the federation to find the accreditation ID
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    let accreditations = &federation.governance.accreditations_to_attest;

    // Find the accreditation we just created
    let user_accreditations = accreditations.get(&receiver_id);
    assert!(user_accreditations.is_some(), "No accreditations found for user");

    let accreditation_id = user_accreditations.unwrap().accreditations.first().unwrap().id.clone();

    // Revoke the accreditation
    let result = client
        .revoke_accreditation_to_attest(*federation_id.object_id(), receiver_id, *accreditation_id.object_id())
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Failed to revoke accreditation to attest: {:?}",
        result.err()
    );

    Ok(())
}

#[tokio::test]
async fn test_revoke_accreditation_to_accredit() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create federation and add property
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let property_name = PropertyName::from("test.revokable.accredit");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("authority".to_string()));

    client
        .add_property(
            *federation_id.object_id(),
            property_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create accreditation to accredit
    let receiver_id = ObjectID::random();
    let property = FederationProperty::new(property_name).with_allowed_values(allowed_values);

    client
        .create_accreditation_to_accredit(*federation_id.object_id(), receiver_id, vec![property])
        .build_and_execute(&client)
        .await?;

    // Get the federation to find the accreditation ID
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    let accreditations = &federation.governance.accreditations_to_accredit;

    // Find the accreditation we just created
    let user_accreditations = accreditations.get(&receiver_id);
    assert!(user_accreditations.is_some(), "No accreditations found for user");

    let accreditation_id = user_accreditations.unwrap().accreditations.first().unwrap().id.clone();

    // Revoke the accreditation
    let result = client
        .revoke_accreditation_to_accredit(*federation_id.object_id(), receiver_id, *accreditation_id.object_id())
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Failed to revoke accreditation to accredit: {:?}",
        result.err()
    );

    Ok(())
}

#[tokio::test]
async fn test_complex_accreditation_workflow() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Add multiple properties with different types
    let verification_name = PropertyName::from("identity.verification");
    let mut verification_values = HashSet::new();
    verification_values.insert(PropertyValue::Text("verified".to_string()));
    verification_values.insert(PropertyValue::Text("pending".to_string()));

    let age_name = PropertyName::from("identity.age");
    let mut age_values = HashSet::new();
    age_values.insert(PropertyValue::Number(18));
    age_values.insert(PropertyValue::Number(21));
    age_values.insert(PropertyValue::Number(25));

    // Add properties to federation
    client
        .add_property(
            *federation_id.object_id(),
            verification_name.clone(),
            verification_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    client
        .add_property(*federation_id.object_id(), age_name.clone(), age_values.clone(), false)
        .build_and_execute(&client)
        .await?;

    // Create multiple properties for accreditation
    let verification_property = FederationProperty::new(verification_name)
        .with_allowed_values(vec![PropertyValue::Text("verified".to_string())]);

    let age_property = FederationProperty::new(age_name).with_allowed_values(vec![PropertyValue::Number(21)]);

    // Create accreditation to attest with multiple properties
    let attestor_id = ObjectID::random();
    client
        .create_accreditation_to_attest(
            *federation_id.object_id(),
            attestor_id,
            vec![verification_property, age_property],
        )
        .build_and_execute(&client)
        .await?;

    // Create accreditation to accredit for a different user
    let accreditor_id = ObjectID::random();
    let accredit_property =
        FederationProperty::new(PropertyName::from("identity.verification")).with_allowed_values(verification_values);

    client
        .create_accreditation_to_accredit(*federation_id.object_id(), accreditor_id, vec![accredit_property])
        .build_and_execute(&client)
        .await?;

    // Verify final federation state
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;

    // Check that both users have their respective accreditations
    assert!(
        federation
            .governance
            .accreditations_to_attest
            .contains_key(&attestor_id)
    );
    assert!(
        federation
            .governance
            .accreditations_to_accredit
            .contains_key(&accreditor_id)
    );

    // Check that properties were properly added
    assert_eq!(federation.governance.properties.data.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_property_with_numeric_values() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Add property with numeric values
    let score_name = PropertyName::from("credit.score");
    let mut score_values = HashSet::new();
    score_values.insert(PropertyValue::Number(600));
    score_values.insert(PropertyValue::Number(700));
    score_values.insert(PropertyValue::Number(800));

    client
        .add_property(
            *federation_id.object_id(),
            score_name.clone(),
            score_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create accreditation with specific numeric value
    let property = FederationProperty::new(score_name).with_allowed_values(vec![PropertyValue::Number(700)]);

    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_attest(*federation_id.object_id(), receiver_id, vec![property])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Failed to create numeric accreditation: {:?}",
        result.err()
    );

    // Verify the accreditation was created
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    assert!(
        federation
            .governance
            .accreditations_to_attest
            .contains_key(&receiver_id)
    );

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_to_accredit_fails_for_nonexistent_property() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Try to create an accreditation for a property that doesn't exist in the federation
    let nonexistent_property_name = PropertyName::from("nonexistent.role");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("admin".to_string()));

    let property = FederationProperty::new(nonexistent_property_name).with_allowed_values(allowed_values);

    // This should fail because the property name doesn't exist in the federation
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_accredit(*federation_id.object_id(), receiver_id, vec![property])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_err(),
        "Expected failure when creating accreditation for nonexistent property, but got success"
    );

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("6"), // EStatementNotInFederation
        "Expected EStatementNotInFederation error, got: {error_msg}"
    );

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_to_attest_fails_for_nonexistent_property() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation (but don't add any properties)
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Try to create an accreditation for a property that doesn't exist in the federation
    let nonexistent_property_name = PropertyName::from("nonexistent.certification");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("verified".to_string()));

    let property = FederationProperty::new(nonexistent_property_name).with_allowed_values(allowed_values);

    // This should fail because the property name doesn't exist in the federation
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_attest(*federation_id.object_id(), receiver_id, vec![property])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_err(),
        "Expected failure when creating accreditation for nonexistent property, but got success"
    );

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("6"), // EPropertyNotInFederation
        "Expected EPropertyNotInFederation error, got: {error_msg}"
    );

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_succeeds_after_adding_property() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // First, add a property to the federation
    let property_name = PropertyName::from("test.role");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("admin".to_string()));
    allowed_values.insert(PropertyValue::Text("user".to_string()));

    client
        .add_property(
            *federation_id.object_id(),
            property_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Now create an accreditation for the property we just added
    let property =
        FederationProperty::new(property_name).with_allowed_values(vec![PropertyValue::Text("admin".to_string())]);

    // This should succeed because the property name exists in the federation
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_accredit(*federation_id.object_id(), receiver_id, vec![property])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Failed to create accreditation for existing property: {:?}",
        result.err()
    );

    // Verify the accreditation was created
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    assert!(
        federation
            .governance
            .accreditations_to_accredit
            .contains_key(&receiver_id)
    );

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_with_multiple_properties_partial_exist() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Add only one property to the federation
    let existing_property_name = PropertyName::from("existing.role");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(PropertyValue::Text("admin".to_string()));

    client
        .add_property(
            *federation_id.object_id(),
            existing_property_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Try to create an accreditation with both existing and non-existing properties
    let existing_property = FederationProperty::new(existing_property_name)
        .with_allowed_values(vec![PropertyValue::Text("admin".to_string())]);

    let nonexistent_property_name = PropertyName::from("nonexistent.certification");
    let nonexistent_property = FederationProperty::new(nonexistent_property_name)
        .with_allowed_values(vec![PropertyValue::Text("verified".to_string())]);

    // This should fail because one of the properties doesn't exist in the federation
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_attest(
            *federation_id.object_id(),
            receiver_id,
            vec![existing_property, nonexistent_property],
        )
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_err(),
        "Expected failure when creating accreditation with partially existing properties, but got success"
    );

    // Check that the error contains the expected error message
    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("6"), // EPropertyNotInFederation
        "Expected EPropertyNotInFederation error, got: {error_msg}"
    );

    Ok(())
}
