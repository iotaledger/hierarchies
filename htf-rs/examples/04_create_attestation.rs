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
  let property_name = TrustedPropertyName::from("Example LTD");

  // Trusted property value
  let value = TrustedPropertyValue::from("Hello");

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
    .create_attestation(federation_id, receiver, vec![constraints], None)
    .await
    .context("Failed to issue permission to attest")?;

  println!("Issued permission to attest");

  // Check if the permission was issued
  let federation: Federation = htf_client.get_object_by_id(federation_id).await?;

  println!("Federation: {:#?}", federation);

  // Check if the receiver has the permission to attest
  let trusted_properties = federation.governance.attesters.contains_key(&receiver);

  assert!(trusted_properties);

  Ok(())
}
