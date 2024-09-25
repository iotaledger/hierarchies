use std::collections::HashSet;

use anyhow::Context;
use examples::get_client;
use htf::types::trusted_constraints::{Timespan, TrustedPropertyConstraint};
use htf::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use htf::types::Federation;
use iota_sdk::types::base_types::ObjectID;

/// Demonstrate how to issue a permission to attest to a trusted property.
///
/// In this example we connect to a locally running private network, but it can
/// be adapted to run on any IOTA node by setting the network and faucet
/// endpoints.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Get the client instance
  let htf_client = get_client().await?;

  // Create new federation
  let federation = htf_client.new_federation(None).await?;

  // Federation ID
  let federation_id = *federation.id.object_id();

  // Trusted property name
  let property_name = TrustedPropertyName::new(vec!["Example LTD".to_string()]);

  // Trusted property value
  let value = TrustedPropertyValue::Text("Hello".to_owned());

  let allowed_values = HashSet::from_iter([value]);

  println!("Adding trusted property");

  // Add the trusted property to the federation
  htf_client
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

  // A receiver is an account that will receive the attestation
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
  htf_client
    .issue_permission_to_attest(federation_id, receiver, vec![constraints.clone()], None)
    .await
    .context("Failed to issue permission to attest")?;

  // Issue permission to the original account
  htf_client
    .issue_permission_to_attest(
      federation_id,
      htf_client.sender_address().into(),
      vec![constraints],
      None,
    )
    .await
    .context("Failed to issue permission to attest")?;

  println!("Issued permission to attest");

  // Check if the permission was issued
  let federation: Federation = htf_client.get_object_by_id(federation_id).await?;

  // Check if the receiver has the permission to attest
  let can_attest = federation.governance.attesters.contains_key(&receiver);

  assert!(can_attest);

  // Revoke the permission
  let permissions = htf_client
    .onchain(federation_id)
    .find_permissions_to_attest(receiver)
    .await
    .context("Failed to find permission to attest")?;

  let permission_id = permissions.permissions[0].id.object_id();

  htf_client
    .revoke_permission_to_attest(federation_id, receiver, *permission_id, None)
    .await
    .context("Failed to revoke permission to attest")?;

  // Check if the permission was revoked
  let federation: Federation = htf_client.get_object_by_id(federation_id).await?;

  println!("Federation: {:#?}", federation);

  // Check if the receiver has the permission to attest
  let can_attest = federation.governance.attesters.get(&receiver).unwrap();

  assert!(can_attest.permissions.is_empty());
  Ok(())
}
