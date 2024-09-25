use std::collections::HashSet;

use anyhow::Context;
use examples::get_client;
use htf::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use htf::types::Federation;

/// Demonstrate how to add a trusted property to a federation.
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
  let another_value = TrustedPropertyValue::Text("World".to_owned());

  let allowed_values = HashSet::from_iter([value, another_value]);

  // Add the trusted property to the federation
  {
    htf_client
      .add_trusted_property(federation_id, property_name.clone(), allowed_values, false, None)
      .await
      .context("Failed to add trusted property")?;
  }

  // Get the updated federation and print it
  let federation: Federation = htf_client.get_object_by_id(federation_id).await?;

  // Check if the trusted property was added
  let trusted_properties = federation
    .governance
    .trusted_constraints
    .contains_property(&property_name);

  assert!(trusted_properties);

  if let Some(constraint) = federation.governance.trusted_constraints.data.get(&property_name) {
    println!("Trusted Property: {:#?}", constraint)
  }

  // Remove the trusted property from the federation
  {
    htf_client
      .remove_trusted_property(federation_id, property_name.clone(), None)
      .await
      .context("Failed to remove trusted property")?;
  }

  // Get the updated federation and print it
  let federation: Federation = htf_client.get_object_by_id(federation_id).await?;

  // Check if the trusted property was removed
  let trusted_properties = federation
    .governance
    .trusted_constraints
    .contains_property(&property_name);

  assert!(!trusted_properties);

  Ok(())
}
