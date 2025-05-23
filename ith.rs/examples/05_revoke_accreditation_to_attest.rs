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
  let client = get_client(urls::localnet::node(), urls::localnet::faucet()).await?;

  // Create new federation
  let federation = client.new_federation(None).await?;

  // Federation ID
  let federation_id = *federation.id.object_id();

  // Trusted property name
  let statement_name = StatementName::from("Example LTD");

  // Trusted property value
  let value = StatementValue::from("Hello");

  let allowed_values = HashSet::from_iter([value]);

  println!("Adding Statement");

  // Add the Statement to the federation
  client
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

  // Property statements
  let statements = Statement {
    statement_name,
    allowed_values,
    expression: None,
    allow_any: false,
    timespan: Timespan::default(),
  };

  // Let us issue a permission to attest to the Statement
  client
    .create_attestation(federation_id, receiver, vec![statements.clone()], None)
    .await
    .context("Failed to issue permission to attest")?;

  // Issue permission to the original account
  client
    .create_attestation(
      federation_id,
      client.sender_address().into(),
      vec![statements],
      None,
    )
    .await
    .context("Failed to issue permission to attest")?;

  println!("Issued permission to attest");

  // Check if the permission was issued
  let federation: Federation = client.get_object_by_id(federation_id).await?;

  // Check if the receiver has the permission to attest
  let can_attest = federation.governance.attesters.contains_key(&receiver);

  assert!(can_attest);

  // Revoke the permission
  let permissions = client
    .onchain(federation_id)
    .get_accreditations_to_attest(receiver)
    .await
    .context("Failed to find permission to attest")?;

  let permission_id = permissions.permissions[0].id.object_id();

  client
    .revoke_accreditation_to_attest(federation_id, receiver, *permission_id, None)
    .await
    .context("Failed to revoke permission to attest")?;

  // Check if the permission was revoked
  let federation: Federation = client.get_object_by_id(federation_id).await?;

  println!("Federation: {:#?}", federation);

  // Check if the receiver has the permission to attest
  let can_attest = federation.governance.attesters.get(&receiver).unwrap();

  assert!(can_attest.permissions.is_empty());
  Ok(())
}
