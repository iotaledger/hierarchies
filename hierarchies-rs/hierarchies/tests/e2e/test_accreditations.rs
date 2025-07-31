// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use iota_interaction::types::base_types::ObjectID;
use hierarchies::core::types::statements::name::StatementName;
use hierarchies::core::types::statements::value::StatementValue;
use hierarchies::core::types::statements::Statement;
use hierarchies::core::types::Federation;

use crate::client::get_funded_test_client;

#[tokio::test]
async fn test_create_accreditation_to_attest() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation and add statements
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let statement_name = StatementName::from("certification.level");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("basic".to_string()));
    allowed_values.insert(StatementValue::Text("advanced".to_string()));

    client
        .add_statement(
            *federation_id.object_id(),
            statement_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create statements for the accreditation
    let mut accreditation_values = HashSet::new();
    accreditation_values.insert(StatementValue::Text("basic".to_string()));

    let statement = Statement::new(statement_name).with_allowed_values(accreditation_values);

    // Create accreditation to attest for a test receiver
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_attest(*federation_id.object_id(), receiver_id, vec![statement])
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

    // Create a new federation and add statements
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let statement_name = StatementName::from("accreditation.authority");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("issuer".to_string()));

    client
        .add_statement(
            *federation_id.object_id(),
            statement_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create statement for the accreditation
    let statement = Statement::new(statement_name).with_allowed_values(allowed_values);

    // Create accreditation to accredit for a test receiver
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_accredit(*federation_id.object_id(), receiver_id, vec![statement])
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

    // Add multiple different statements
    let statements_to_add = vec![
        (
            StatementName::from("identity.verification.level"),
            vec![
                StatementValue::Text("basic".to_string()),
                StatementValue::Text("enhanced".to_string()),
            ],
            false,
        ),
        (
            StatementName::from("identity.age.minimum"),
            vec![StatementValue::Number(18), StatementValue::Number(21)],
            false,
        ),
        (
            StatementName::from("identity.custom.field"),
            vec![], // Empty for allow_any
            true,
        ),
    ];

    // Add all statements
    for (name, values, allow_any) in statements_to_add.iter() {
        let allowed_values: HashSet<StatementValue> = values.iter().cloned().collect();
        client
            .add_statement(*federation_id.object_id(), name.clone(), allowed_values, *allow_any)
            .build_and_execute(&client)
            .await?;
    }

    // Verify all statements were added
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    let statements = &federation.governance.statements.data;

    assert_eq!(statements.len(), statements_to_add.len());

    for (name, values, allow_any) in statements_to_add {
        assert!(statements.contains_key(&name), "Statement {name:?} not found");
        let statement = statements.get(&name).unwrap();
        assert_eq!(statement.allow_any, allow_any, "Allow any mismatch for {name:?}");

        if !allow_any {
            let expected_values: HashSet<StatementValue> = values.iter().cloned().collect();
            assert_eq!(
                statement.allowed_values, expected_values,
                "Values mismatch for {name:?}"
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_revoke_accreditation_to_attest() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create federation and add statement
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let statement_name = StatementName::from("test.revokable");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("value".to_string()));

    client
        .add_statement(
            *federation_id.object_id(),
            statement_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create accreditation to attest
    let receiver_id = ObjectID::random();
    let statement = Statement::new(statement_name).with_allowed_values(allowed_values);

    client
        .create_accreditation_to_attest(*federation_id.object_id(), receiver_id, vec![statement])
        .build_and_execute(&client)
        .await?;

    // Get the federation to find the permission ID
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    let accreditations = &federation.governance.accreditations_to_attest;

    // Find the accreditation we just created
    let user_accreditations = accreditations.get(&receiver_id);
    assert!(user_accreditations.is_some(), "No accreditations found for user");

    let permission_id = user_accreditations.unwrap().statements.first().unwrap().id.clone();

    // Revoke the accreditation
    let result = client
        .revoke_accreditation_to_attest(*federation_id.object_id(), receiver_id, *permission_id.object_id())
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

    // Create federation and add statement
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let statement_name = StatementName::from("test.revokable.accredit");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("authority".to_string()));

    client
        .add_statement(
            *federation_id.object_id(),
            statement_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create accreditation to accredit
    let receiver_id = ObjectID::random();
    let statement = Statement::new(statement_name).with_allowed_values(allowed_values);

    client
        .create_accreditation_to_accredit(*federation_id.object_id(), receiver_id, vec![statement])
        .build_and_execute(&client)
        .await?;

    // Get the federation to find the permission ID
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    let accreditations = &federation.governance.accreditations_to_accredit;

    // Find the accreditation we just created
    let user_accreditations = accreditations.get(&receiver_id);
    assert!(user_accreditations.is_some(), "No accreditations found for user");

    let permission_id = user_accreditations.unwrap().statements.first().unwrap().id.clone();

    // Revoke the accreditation
    let result = client
        .revoke_accreditation_to_accredit(*federation_id.object_id(), receiver_id, *permission_id.object_id())
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

    // Add multiple statements with different types
    let verification_name = StatementName::from("identity.verification");
    let mut verification_values = HashSet::new();
    verification_values.insert(StatementValue::Text("verified".to_string()));
    verification_values.insert(StatementValue::Text("pending".to_string()));

    let age_name = StatementName::from("identity.age");
    let mut age_values = HashSet::new();
    age_values.insert(StatementValue::Number(18));
    age_values.insert(StatementValue::Number(21));
    age_values.insert(StatementValue::Number(25));

    // Add statements to federation
    client
        .add_statement(
            *federation_id.object_id(),
            verification_name.clone(),
            verification_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    client
        .add_statement(*federation_id.object_id(), age_name.clone(), age_values.clone(), false)
        .build_and_execute(&client)
        .await?;

    // Create multiple statements for accreditation
    let verification_statement =
        Statement::new(verification_name).with_allowed_values(vec![StatementValue::Text("verified".to_string())]);

    let age_statement = Statement::new(age_name).with_allowed_values(vec![StatementValue::Number(21)]);

    // Create accreditation to attest with multiple statements
    let attestor_id = ObjectID::random();
    client
        .create_accreditation_to_attest(
            *federation_id.object_id(),
            attestor_id,
            vec![verification_statement, age_statement],
        )
        .build_and_execute(&client)
        .await?;

    // Create accreditation to accredit for a different user
    let accreditor_id = ObjectID::random();
    let accredit_statement =
        Statement::new(StatementName::from("identity.verification")).with_allowed_values(verification_values);

    client
        .create_accreditation_to_accredit(*federation_id.object_id(), accreditor_id, vec![accredit_statement])
        .build_and_execute(&client)
        .await?;

    // Verify final federation state
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;

    // Check that both users have their respective accreditations
    assert!(federation
        .governance
        .accreditations_to_attest
        .contains_key(&attestor_id));
    assert!(federation
        .governance
        .accreditations_to_accredit
        .contains_key(&accreditor_id));

    // Check that statements were properly added
    assert_eq!(federation.governance.statements.data.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_statement_with_numeric_values() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Add statement with numeric values
    let score_name = StatementName::from("credit.score");
    let mut score_values = HashSet::new();
    score_values.insert(StatementValue::Number(600));
    score_values.insert(StatementValue::Number(700));
    score_values.insert(StatementValue::Number(800));

    client
        .add_statement(
            *federation_id.object_id(),
            score_name.clone(),
            score_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Create accreditation with specific numeric value
    let statement = Statement::new(score_name).with_allowed_values(vec![StatementValue::Number(700)]);

    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_attest(*federation_id.object_id(), receiver_id, vec![statement])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Failed to create numeric accreditation: {:?}",
        result.err()
    );

    // Verify the accreditation was created
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    assert!(federation
        .governance
        .accreditations_to_attest
        .contains_key(&receiver_id));

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_to_accredit_fails_for_nonexistent_statement() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Try to create an accreditation for a statement that doesn't exist in the federation
    let nonexistent_statement_name = StatementName::from("nonexistent.role");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("admin".to_string()));

    let statement = Statement::new(nonexistent_statement_name).with_allowed_values(allowed_values);

    // This should fail because the statement name doesn't exist in the federation
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_accredit(*federation_id.object_id(), receiver_id, vec![statement])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_err(),
        "Expected failure when creating accreditation for nonexistent statement, but got success"
    );

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("7"), // EStatementNotInFederation
        "Expected EStatementNotInFederation error, got: {error_msg}"
    );

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_to_attest_fails_for_nonexistent_statement() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation (but don't add any statements)
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Try to create an accreditation for a statement that doesn't exist in the federation
    let nonexistent_statement_name = StatementName::from("nonexistent.certification");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("verified".to_string()));

    let statement = Statement::new(nonexistent_statement_name).with_allowed_values(allowed_values);

    // This should fail because the statement name doesn't exist in the federation
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_attest(*federation_id.object_id(), receiver_id, vec![statement])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_err(),
        "Expected failure when creating accreditation for nonexistent statement, but got success"
    );

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("7"), // EStatementNotInFederation
        "Expected EStatementNotInFederation error, got: {error_msg}"
    );

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_succeeds_after_adding_statement() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // First, add a statement to the federation
    let statement_name = StatementName::from("test.role");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("admin".to_string()));
    allowed_values.insert(StatementValue::Text("user".to_string()));

    client
        .add_statement(
            *federation_id.object_id(),
            statement_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Now create an accreditation for the statement we just added
    let statement = Statement::new(statement_name).with_allowed_values(vec![StatementValue::Text("admin".to_string())]);

    // This should succeed because the statement name exists in the federation
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_accredit(*federation_id.object_id(), receiver_id, vec![statement])
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_ok(),
        "Failed to create accreditation for existing statement: {:?}",
        result.err()
    );

    // Verify the accreditation was created
    let federation: Federation = client.get_federation_by_id(*federation_id.object_id()).await?;
    assert!(federation
        .governance
        .accreditations_to_accredit
        .contains_key(&receiver_id));

    Ok(())
}

#[tokio::test]
async fn test_create_accreditation_with_multiple_statements_partial_exist() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Add only one statement to the federation
    let existing_statement_name = StatementName::from("existing.role");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("admin".to_string()));

    client
        .add_statement(
            *federation_id.object_id(),
            existing_statement_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Try to create an accreditation with both existing and non-existing statements
    let existing_statement =
        Statement::new(existing_statement_name).with_allowed_values(vec![StatementValue::Text("admin".to_string())]);

    let nonexistent_statement_name = StatementName::from("nonexistent.certification");
    let nonexistent_statement = Statement::new(nonexistent_statement_name)
        .with_allowed_values(vec![StatementValue::Text("verified".to_string())]);

    // This should fail because one of the statements doesn't exist in the federation
    let receiver_id = ObjectID::random();
    let result = client
        .create_accreditation_to_attest(
            *federation_id.object_id(),
            receiver_id,
            vec![existing_statement, nonexistent_statement],
        )
        .build_and_execute(&client)
        .await;

    assert!(
        result.is_err(),
        "Expected failure when creating accreditation with partially existing statements, but got success"
    );

    // Check that the error contains the expected error message
    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("7"), // EStatementNotInFederation
        "Expected EStatementNotInFederation error, got: {error_msg}"
    );

    Ok(())
}
