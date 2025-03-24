use std::collections::HashSet;

use anyhow::Context;
use examples::{get_client, urls};
use iota_sdk::types::base_types::ObjectID;
use ith::types::{Timespan, TrustedPropertyConstraint};
use ith::types::{TrustedPropertyName, TrustedPropertyValue};

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

  let attestations = ith_client
    .offchain(*federation_id)
    .await?
    .get_attestations(user_id);

  println!("Permissions: {:#?}", attestations);

  //   Add trusted property
  let property_name = TrustedPropertyName::from("Example LTD");
  let value = TrustedPropertyValue::from("Hello");
  let allowed_values = HashSet::from_iter([value]);

  ith_client
    .add_trusted_property(
      *federation_id,
      property_name.clone(),
      allowed_values.clone(),
      false,
      None,
    )
    .await
    .context("Failed to add trusted property")?;

  // Add new receiver
  let receiver = ObjectID::random();

  // Property constraints
  let constraints = TrustedPropertyConstraint {
    property_name,
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

  // Check if the permission was issued
  let attestations = ith_client
    .offchain(*federation_id)
    .await?
    .get_attestations(receiver)
    .context("Failed to find permission to attest")?;

  assert!(attestations.permissions.len() == 1);

  println!("Permissions: {:#?}", attestations);

  // Issue Accredit permission
  {
    ith_client
      .create_accreditation(*federation_id, receiver, vec![constraints], None)
      .await
      .context("Failed to issue permission to accredit")?;
  }

  // Check if the permission was issued
  let accreditations = ith_client
    .offchain(*federation_id)
    .await?
    .get_accreditations(receiver)
    .context("Failed to find permission to accredit")?;

  assert!(accreditations.permissions.len() == 1);

  Ok(())
}
