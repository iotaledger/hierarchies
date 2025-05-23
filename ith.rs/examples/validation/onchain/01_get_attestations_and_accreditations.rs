use std::collections::HashSet;

use anyhow::Context;
use examples::{get_client, urls};
use iota_sdk::types::base_types::ObjectID;
use ith::types::{Statement, Timespan};
use ith::types::{StatementName, StatementValue};

/// Demonstrates how to use the offchain API to check if a user has a permission to attest and accredit.
///
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

  let user_id = ith_client.sender_address().into();

  let permissions = ith_client
    .offchain(*federation_id)
    .await?
    .get_accreditations_to_attest(user_id);

  println!("Accreditations: {:#?}", permissions);

  //   Add Statement
  let statement_name = StatementName::from("Example LTD");
  let value = StatementValue::from("Hello");
  let allowed_values = HashSet::from_iter([value]);

  ith_client
    .add_statement(
      *federation_id,
      statement_name.clone(),
      allowed_values.clone(),
      false,
      None,
    )
    .await
    .context("Failed to add Statement")?;

  // Add new receiver
  let receiver = ObjectID::random();

  // Statements
  let statements = Statement {
    statement_name,
    allowed_values,
    expression: None,
    allow_any: false,
    timespan: Timespan::default(),
  };

  // Let us issue a permission to attest to the Statement
  {
    ith_client
      .create_attestation(*federation_id, receiver, vec![statements.clone()], None)
      .await
      .context("Failed to issue permission to attest")?;
  }

  // Check if the permission was issued
  let permissions = ith_client
    .onchain(*federation_id)
    .get_accreditations_to_attest(receiver)
    .await
    .context("Failed to find permission to attest")?;

  assert!(permissions.permissions.len() == 1);

  println!("Accreditations: {:#?}", permissions);

  // Issue Accredit permission
  {
    ith_client
      .create_accreditation(*federation_id, receiver, vec![statements], None)
      .await
      .context("Failed to issue permission to accredit")?;
  }

  // Check if the permission was issued
  let permissions = ith_client
    .onchain(*federation_id)
    .get_accreditations_to_accredit(receiver)
    .await
    .context("Failed to find permission to accredit")?;

  assert!(permissions.permissions.len() == 1);

  Ok(())
}
