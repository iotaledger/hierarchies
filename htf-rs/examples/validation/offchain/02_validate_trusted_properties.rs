use std::collections::{HashMap, HashSet};

use anyhow::Context;
use examples::get_client;
use htf::types::trusted_constraints::TrustedPropertyConstraint;
use htf::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use iota_sdk::types::base_types::ObjectID;

/// Demonstrates how to use the offchain API to validate trusted properties.
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
  let property_name = TrustedPropertyName::new(vec!["Example LTD".to_string()]);
  let value = TrustedPropertyValue::Text("Hello".to_owned());
  let allowed_values = HashSet::from_iter([value.clone()]);

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

  // Add new receiver
  let receiver = ObjectID::random();

  // Property constraints
  let constraints = TrustedPropertyConstraint {
    property_name: property_name.clone(),
    allowed_values,
    expression: None,
    allow_any: false,
  };

  // Let us issue a permission to attest to the trusted property
  {
    htf_client
      .issue_permission_to_attest(*federation_id, receiver, vec![constraints.clone()], None)
      .await
      .context("Failed to issue permission to attest")?;
  }

  // Validate trusted properties
  let trusted_properties = HashMap::from_iter([(property_name, value)]);

  let validate = htf_client
    .offchain(*federation_id)
    .await?
    .validate_trusted_properties((*receiver).into(), trusted_properties)
    .context("Failed to validate trusted properties")?;

  assert!(validate);

  println!("Validated trusted properties");

  Ok(())
}
