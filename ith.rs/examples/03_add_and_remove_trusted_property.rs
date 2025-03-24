use anyhow::Context;
use examples::{get_client, urls};
use ith::types::Federation;
use ith::types::{TrustedPropertyName, TrustedPropertyValue};

/// Demonstrate how to add a trusted property to a federation.
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
  let another_value = TrustedPropertyValue::from("World");
  let allowed_values = [value, another_value];

  // Add the trusted property to the federation
  {
    ith_client
      .add_trusted_property(
        federation_id,
        property_name.clone(),
        allowed_values,
        false,
        None,
      )
      .await
      .context("Failed to add trusted property")?;
  }

  // Get the updated federation and print it
  let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

  // Check if the trusted property was added
  let trusted_properties = federation
    .governance
    .trusted_constraints
    .contains_property(&property_name);

  assert!(trusted_properties);

  if let Some(constraint) = federation
    .governance
    .trusted_constraints
    .data
    .get(&property_name)
  {
    println!("Trusted Property: {:#?}", constraint)
  }

  // Remove the trusted property from the federation
  {
    ith_client
      .remove_trusted_property(federation_id, property_name.clone(), None)
      .await
      .context("Failed to remove trusted property")?;
  }

  // Get the updated federation and print it
  let federation: Federation = ith_client.get_object_by_id(federation_id).await?;

  // Check if the trusted property was removed
  let trusted_properties = federation
    .governance
    .trusted_constraints
    .contains_property(&property_name);

  assert!(!trusted_properties);

  Ok(())
}
