use std::collections::HashSet;

use anyhow::Context;
use examples::{get_client, urls};
use iota_sdk::types::base_types::ObjectID;
use ith::types::{Statement, Timespan};
use ith::types::{StatementName, StatementValue};

/// Demonstrates how to use the offchain API to validate trusted properties.
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.md

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let ith_client = get_client(urls::localnet::node(), urls::localnet::faucet()).await?;

  let federation = ith_client.new_federation(None).await?;
  let federation_id = federation.id.object_id();

  //   Add trusted property
  let statement_name = StatementName::from("Example LTD");
  let value = StatementValue::from("Hello");
  let allowed_values = HashSet::from_iter([value.clone()]);

  ith_client
    .add_trusted_statement(
      *federation_id,
      statement_name.clone(),
      allowed_values.clone(),
      false,
      None,
    )
    .await
    .context("Failed to add trusted property")?;

  // Add new receiver
  let receiver = ObjectID::random();

  // Property constraints
  let constraints = Statement {
    statement_name: statement_name.clone(),
    allowed_values,
    expression: None,
    allow_any: false,
    timespan: Timespan::default(),
  };

  // Let us issue a permission to attest to the trusted property
  {
    ith_client
      .create_attestation(*federation_id, receiver, vec![constraints.clone()], None)
      .await
      .context("Failed to issue permission to attest")?;
  }

  // Validate trusted properties
  let trusted_statements = [(statement_name, value)];

  let validate = ith_client
    .onchain(*federation_id)
    .validatestatements((*receiver).into(), trusted_statements)
    .await;

  assert!(validate.is_ok());

  println!("Validated trusted properties");

  Ok(())
}
