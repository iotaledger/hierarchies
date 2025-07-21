use std::collections::HashSet;

use ith::client::get_object_ref_by_id_with_bcs;
use ith::core::types::statements::name::StatementName;
use ith::core::types::statements::value::StatementValue;
use ith::core::types::Federation;
use product_common::core_client::{CoreClient, CoreClientReadOnly};

use crate::client::{get_funded_test_client, TestClient};

/// Helper function to create a federation for testing purposes.
/// Returns the federation object and transaction response.
async fn create_test_federation() -> anyhow::Result<(Federation, TestClient)> {
    let client = get_funded_test_client().await?;

    let federation_result = client.create_new_federation().build_and_execute(&client).await?;

    let federation = federation_result.output;

    Ok((federation, client))
}

#[tokio::test]
async fn test_add_statement() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation first
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Create a statement name and allowed values
    let statement_name = StatementName::from("test.credential.type");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("verified".to_string()));
    allowed_values.insert(StatementValue::Text("pending".to_string()));

    // Add the statement to the federation
    let result = client
        .add_statement(
            *federation_id.object_id(),
            statement_name.clone(),
            allowed_values.clone(),
            false,
        )
        .build_and_execute(&client)
        .await;

    assert!(result.is_ok(), "Failed to add statement: {:?}", result.err());

    // Verify the statement was added by fetching the federation
    let federation: Federation = get_object_ref_by_id_with_bcs(&client, federation_id.object_id()).await?;
    let statements = &federation.governance.statements.data;

    assert!(
        statements.contains_key(&statement_name),
        "Statement not found in federation"
    );
    let added_statement = statements.get(&statement_name).unwrap();
    assert_eq!(added_statement.allowed_values, allowed_values);

    Ok(())
}

#[tokio::test]
async fn test_revoke_statement() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation and add a statement
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    let statement_name = StatementName::from("test.temporary.credential");
    let mut allowed_values = HashSet::new();
    allowed_values.insert(StatementValue::Text("active".to_string()));

    // Add the statement
    client
        .add_statement(
            *federation_id.object_id(),
            statement_name.clone(),
            allowed_values,
            false,
        )
        .build_and_execute(&client)
        .await?;
    let result = client
        .revoke_statement(*federation_id.object_id(), statement_name.clone(), None)
        .build_and_execute(&client)
        .await;

    assert!(result.is_ok(), "Failed to revoke statement: {:?}", result.err());

    Ok(())
}

#[tokio::test]
async fn test_create_and_get_statements() -> anyhow::Result<()> {
    let (federation, client) = create_test_federation().await?;

    let statement_name = StatementName::new(vec!["test_statement"]);

    let statement_values = HashSet::from_iter([StatementValue::Text("test_value".to_string())]);

    client
        .add_statement(
            *federation.id.object_id(),
            statement_name.clone(),
            statement_values,
            false,
        )
        .build_and_execute(&client)
        .await?;

    // Get statements
    let statements = client.get_statements(*federation.id.object_id()).await?;

    assert_eq!(statements.len(), 1);

    let statement = statements.first().unwrap();
    assert_eq!(statement.clone(), statement_name);

    Ok(())
}

#[tokio::test]
async fn test_create_and_validate_statement() -> anyhow::Result<()> {
    let (federation, client) = create_test_federation().await?;

    let statement_name = StatementName::new(vec!["test_statement"]);

    let statement_value = StatementValue::Text("test_value".to_string());

    let statement_values = HashSet::from_iter([statement_value.clone()]);
    client
        .add_statement(
            *federation.id.object_id(),
            statement_name.clone(),
            statement_values,
            false,
        )
        .build_and_execute(&client)
        .await?;

    client
        .validate_statement(
            *federation.id.object_id(),
            client.sender_address().into(),
            statement_name.clone(),
            statement_value,
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_add_statement_with_allow_any() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    // Create a new federation
    let federation_id = client
        .create_new_federation()
        .build_and_execute(&client)
        .await?
        .output
        .id;

    // Create a statement that allows any value
    let statement_name = StatementName::from("test.open.field");
    let allowed_values = HashSet::new(); // Empty set when allow_any is true

    let result = client
        .add_statement(*federation_id.object_id(), statement_name.clone(), allowed_values, true)
        .build_and_execute(&client)
        .await;

    assert!(result.is_ok(), "Failed to add allow-any statement: {:?}", result.err());

    // Verify the statement was added
    let federation: Federation = client.get_object_by_id(*federation_id.object_id()).await?;
    let statements = &federation.governance.statements.data;

    assert!(statements.contains_key(&statement_name));
    let added_statement = statements.get(&statement_name).unwrap();
    assert!(added_statement.allow_any, "Statement should allow any value");

    Ok(())
}
