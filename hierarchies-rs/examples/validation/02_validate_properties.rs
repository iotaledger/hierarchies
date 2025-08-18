// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use anyhow::Context;
use hierarchies::core::types::property::FederationProperty;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::property_value::PropertyValue;
use hierarchies::core::types::timespan::Timespan;
use hierarchies_examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;

/// Demonstrates how to use the offchain API to validate properties.
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
    let property_name = PropertyName::from("Example LTD");
    let value = PropertyValue::Text("Hello".to_owned());
    let allowed_values = HashSet::from_iter([value.clone()]);

    hierarchies_client
        .add_property(*federation_id, property_name.clone(), allowed_values.clone(), false)
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to add Property")?;

    // Add new receiver
    let receiver = ObjectID::random();

    // Properties
    let properties = FederationProperty {
        name: property_name.clone(),
        allowed_values,
        shape: None,
        allow_any: false,
        timespan: Timespan::default(),
    };

    // Let us issue a permission to attest to the Property
    {
        hierarchies_client
            .create_accreditation_to_attest(*federation_id, receiver, vec![properties.clone()])
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to issue permission to attest")?;
    }

    // Validate if properties can be attested by the receiver
    let properties = [(property_name, value)];

    let validate = hierarchies_client
        .validate_properties(*federation_id, (*receiver).into(), properties)
        .await;

    assert!(validate.is_ok());

    println!("Validated attestations");

    Ok(())
}
