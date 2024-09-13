use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use anyhow::Context;
use examples::get_client;
use htf::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use htf::types::Federation;
use iota_sdk::types::base_types::ObjectID;

/// Demonstrate how to issue a credential to a federation.
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

  println!("Adding trusted property");
  // Add the trusted property to the federation
  htf_client
    .add_trusted_property(
      federation_id,
      property_name.clone(),
      HashSet::from_iter([value.clone()]),
      false,
      None,
    )
    .await
    .context("Failed to add trusted property")?;

  println!("Trusted Property: {:#?}", property_name);

  let trusted_properties = HashMap::from_iter([(property_name, value)]);

  let bob_id = ObjectID::from_single_byte(5);

  let now_ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();

  let valid_until_ts = now_ts + 3_600_000;

  // Issue a credential
  htf_client
    .issue_credential(federation_id, bob_id, trusted_properties, now_ts, valid_until_ts, None)
    .await
    .context("Failed to add trusted property")?;

  println!("Issued credential");

  // Get the updated federation and print it
  let federation: Federation = htf_client.get_object_by_id(federation_id).await?;

  // print!("Trusted Properties : {:#?}", federation.governance.trusted_constraints);

  // // Check if the trusted property was added
  // let trusted_properties = federation
  //   .governance
  //   .trusted_constraints
  //   .contains_property(&property_name);

  // assert!(trusted_properties);

  // if let Some(constraint) = federation.governance.trusted_constraints.data.get(&property_name) {
  //   println!("Trusted Property: {:#?}", constraint)
  // }

  Ok(())
}
