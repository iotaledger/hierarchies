use std::collections::HashSet;

use anyhow::Context;
use hierarchies::core::types::Federation;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::value::PropertyValue;
use hierarchies_examples::get_funded_client;

/// Demonstrate how to add a Statement to a federation.
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
    let hierarchies_client = get_funded_client().await?;

    // Create new federation
    let federation = hierarchies_client
        .create_new_federation()
        .build_and_execute(&hierarchies_client)
        .await?;

    // Federation ID
    let federation_id = *federation.output.id.object_id();

    // Federation property name
    let property_name = PropertyName::from("Example LTD");

    // Federation property value
    let value = PropertyValue::Text("Hello".to_owned());
    let another_value = PropertyValue::Text("World".to_owned());
    let allowed_values = HashSet::from([value, another_value]);

    // Add the Property to the federation
    {
        hierarchies_client
            .add_property(federation_id, property_name.clone(), allowed_values, false)
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to add Property")?;
    }

    // Get the updated federation and print it
    let federation: Federation = hierarchies_client.get_federation_by_id(federation_id).await?;

    // Check if the Property was added
    let federation_properties = federation.governance.properties.data.contains_key(&property_name);

    assert!(federation_properties);

    if let Some(property) = federation.governance.properties.data.get(&property_name) {
        println!("Trusted Property: {property:#?}")
    }

    Ok(())
}
