use std::collections::HashSet;

use anyhow::Context;
use examples::{get_client, urls};
use iota_sdk::types::base_types::ObjectID;
use ith::types::Federation;
use ith::types::{Statement, Timespan};
use ith::types::{StatementName, StatementValue};

/// Demonstrate how to issue a permission to attest to a Statement.
///
/// In this example we connect to a locally running private network, but it can
/// be adapted to run on any IOTA node by setting the network and faucet
/// endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.mdx

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Get the client instance
  let ith_client = get_client(urls::localnet::node(), urls::localnet::faucet()).await?;

  // Create new federation
  let federation = ith_client.new_federation(None).await?;

  // Federation ID
  let federation_id = *federation.id.object_id();

  // Trusted property name
  let statement_name = StatementName::from("Example LTD");

  // Trusted property value
  let value = StatementValue::from("Hello");

  let allowed_values = HashSet::from_iter([value]);

  println!("Adding Statement");

  // Add the Statement to the federation
  ith_client
    .add_statement(
      federation_id,
      statement_name.clone(),
      allowed_values.clone(),
      false,
      None,
    )
    .await
    .context("Failed to add Statement")?;

  println!("Added Statement");

  // A receiver is an account that will receive the attestation
  let receiver = ObjectID::random();

  // Statements
  let statements = Statement {
    statement_name,
    allowed_values,
    condition: None,
    allow_any: false,
    timespan: Timespan::default(),
  };

  // Let us issue a permission to attest to the Statement
  ith_client
    .create_accreditation_to_attest(federation_id, receiver, vec![statements], None)
    .await
    .context("Failed to issue permission to attest")?;

  println!("Issued permission to attest");

  // Check if the permission was issued
  let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

  println!("Federation: {:#?}", federation);

  // Check if the receiver has the permission to attest
  let statements = federation
    .governance
    .accreditations_to_attest
    .contains_key(&receiver);

  assert!(statements);

  Ok(())
}
