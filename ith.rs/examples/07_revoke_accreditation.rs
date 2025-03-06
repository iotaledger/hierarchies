use std::collections::HashSet;

use anyhow::Context;
use examples::{get_client, urls};
use iota_sdk::types::base_types::ObjectID;
use ith::types::Federation;
use ith::types::{Timespan, TrustedPropertyConstraint};
use ith::types::{TrustedPropertyName, TrustedPropertyValue};

/// Demonstrate how to issue a permission to accredit to a trusted property.
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
  let property_name = TrustedPropertyName::from("Example LTD");

  // Trusted property value
  let value = TrustedPropertyValue::from("Hello");

  let allowed_values = HashSet::from_iter([value]);

  println!("Adding trusted property");

  // Add the trusted property to the federation
  ith_client
    .add_trusted_property(
      federation_id,
      property_name.clone(),
      allowed_values.clone(),
      false,
      None,
    )
    .await
    .context("Failed to add trusted property")?;

  println!("Added trusted property");

  // A receiver is an account that will receive the accreditation
  let receiver = ObjectID::random();

  // Property constraints
  let constraints = TrustedPropertyConstraint {
    property_name,
    allowed_values,
    expression: None,
    allow_any: false,
    timespan: Timespan::default(),
  };

  // Let us issue a permission to accredit to the trusted property
  ith_client
    .create_accreditation(federation_id, receiver, vec![constraints.clone()], None)
    .await
    .context("Failed to issue permission to attest")?;

  // Issue permission to the original account
  ith_client
    .create_accreditation(
      federation_id,
      ith_client.sender_address().into(),
      vec![constraints],
      None,
    )
    .await
    .context("Failed to issue permission to attest")?;

  println!("Issued permission to attest");

  // Check if the permission was issued
  let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

  // Check if the receiver has the permission to accredit
  let can_accredit = federation.governance.accreditors.contains_key(&receiver);

  assert!(can_accredit);

  // Revoke the permission
  let permissions = ith_client
    .onchain(federation_id)
    .get_accreditations(receiver)
    .await
    .context("Failed to find permission to accredit")?;

  let permission_id = permissions.permissions[0].id.object_id();

  ith_client
    .revoke_accreditation(federation_id, receiver, *permission_id, None)
    .await
    .context("Failed to revoke permission to accredit")?;

  // Check if the permission was revoked
  let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

  println!("Federation: {:#?}", federation);

  // Check if the receiver has the permission to accredit
  let can_accredit = federation.governance.accreditors.get(&receiver).unwrap();

  assert!(can_accredit.permissions.is_empty());
  Ok(())
}
