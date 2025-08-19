// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use anyhow::Context;
use hierarchies::core::types::property::FederationProperty;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::property_value::PropertyValue;
use hierarchies_examples::get_funded_client;

/// Demonstrates how to use the offchain API to get federation properties.
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.md

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hierarchies_client = get_funded_client().await?;

    let federation = hierarchies_client
        .create_new_federation()
        .build_and_execute(&hierarchies_client)
        .await?;
    let federation_id = federation.output.id.object_id();

    //   Add Property
    {
        let property_name = PropertyName::from("Example LTD");
        let value = PropertyValue::Text("Hello".to_owned());
        let allowed_values: HashSet<PropertyValue> = HashSet::from_iter([value]);

        // Add the Property to the federation
        hierarchies_client
            .add_property(
                *federation_id,
                FederationProperty::new(property_name.clone()).with_allowed_values(allowed_values.clone()),
            )
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to add Property")?;
    }

    // Add second Property
    {
        let property_name = PropertyName::new(["Example LTD 2", "Example LTD 3"]);
        let value = PropertyValue::Text("Hello 2".to_owned());
        let allowed_values: HashSet<PropertyValue> = HashSet::from_iter([value]);

        // Add the Property to the federation
        hierarchies_client
            .add_property(
                *federation_id,
                FederationProperty::new(property_name.clone()).with_allowed_values(allowed_values.clone()),
            )
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to add Property")?;
    }

    let federation_properties = hierarchies_client.get_properties(*federation_id).await?;

    assert!(federation_properties.len() == 2);

    println!("Federation properties: {federation_properties:#?}");

    Ok(())
}
