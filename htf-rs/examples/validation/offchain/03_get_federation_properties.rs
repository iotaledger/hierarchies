use std::collections::HashSet;

use anyhow::Context;
use examples::get_client;
use htf::types::trusted_constraints::TrustedPropertyConstraint;
use htf::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use iota_sdk::types::base_types::ObjectID;

/// Demonstrates how to use the offchain API to get federation properties.
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.md
///
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let htf_client = get_client().await?;

  let federation = htf_client.new_federation(None).await?;
  let federation_id = federation.id.object_id();

  //   Add trusted property
  {
    let property_name = TrustedPropertyName::new(vec!["Example LTD".to_string()]);
    let value = TrustedPropertyValue::Text("Hello".to_owned());
    let allowed_values = HashSet::from_iter([value.clone()]);

    // Add the trusted property to the federation
    htf_client
      .add_trusted_property(
        *federation_id,
        property_name.clone(),
        allowed_values.clone(),
        false,
        None,
      )
      .await
      .context("Failed to add trusted property")?;
  }

  // Add second trusted property
  {
    let property_name = TrustedPropertyName::new(vec!["Example LTD 2".to_string(), "Example LTD 3".to_string()]);
    let value = TrustedPropertyValue::Text("Hello 2".to_owned());
    let allowed_values = HashSet::from_iter([value.clone()]);

    // Add the trusted property to the federation
    htf_client
      .add_trusted_property(
        *federation_id,
        property_name.clone(),
        allowed_values.clone(),
        false,
        None,
      )
      .await
      .context("Failed to add trusted property")?;
  }

  let federation_properties = htf_client.offchain(*federation_id).await?.get_federation_properties();

  assert!(federation_properties.len() == 2);

  println!("Federation properties: {:#?}", federation_properties);

  Ok(())
}
