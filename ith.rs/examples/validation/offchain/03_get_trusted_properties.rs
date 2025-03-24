use anyhow::Context;
use examples::{get_client, urls};
use ith::types::{TrustedPropertyName, TrustedPropertyValue};

/// Demonstrates how to use the offchain API to get federation properties.
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

  //   Add trusted property
  {
    let property_name = TrustedPropertyName::from("Example LTD");
    let value = TrustedPropertyValue::from("Hello");
    let allowed_values = [value];

    // Add the trusted property to the federation
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
  }

  // Add second trusted property
  {
    let property_name = TrustedPropertyName::new(["Example LTD 2", "Example LTD 3"]);
    let value = TrustedPropertyValue::from("Hello 2");
    let allowed_values = [value.clone()];

    // Add the trusted property to the federation
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
  }

  let federation_properties = ith_client
    .offchain(*federation_id)
    .await?
    .get_trusted_properties();

  assert!(federation_properties.len() == 2);

  println!("Federation properties: {:#?}", federation_properties);

  Ok(())
}
