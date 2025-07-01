use std::collections::HashSet;

use crate::client::{get_funded_test_client, TestClient};
use ith::core::types::statements::name::StatementName;
use ith::core::types::statements::value::StatementValue;
use ith::core::types::Federation;
use product_common::core_client::CoreClient;

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
    let (federation, client) = create_test_federation().await?;

    let statement_name = StatementName::new(vec!["test_statement"]);

    let statement_values = HashSet::from_iter([StatementValue::Text("test_value".to_string())]);

    let tx = client
        .add_statement(*federation.id.object_id(), statement_name, statement_values, false)
        .build_and_execute(&client)
        .await;

    assert!(tx.is_ok());

    Ok(())
}

#[tokio::test]
#[ignore = "revoke statement is not implemented"]
async fn test_revoke_statement() -> anyhow::Result<()> {
    let (federation, client) = create_test_federation().await?;

    let statement_name = StatementName::new(vec!["test_statement"]);

    let statement_values = HashSet::from_iter([StatementValue::Text("test_value".to_string())]);

    let tx = client
        .add_statement(*federation.id.object_id(), statement_name, statement_values, false)
        .build_and_execute(&client)
        .await?;

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

    let statement = client
        .validate_statement(
            *federation.id.object_id(),
            client.sender_address().into(),
            statement_name.clone(),
            statement_value,
        )
        .await?;
    assert!(statement, "Statement should be valid");

    Ok(())
}
