// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use anyhow::Context;
use hierarchies::core::types::Federation;
use hierarchies::core::types::property::FederationProperty;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::property_value::PropertyValue;
use hierarchies::core::types::timespan::Timespan;
use hierarchies_examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;

/// Demonstrate how to issue an accreditation to attest to a Property.
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

    let allowed_values = HashSet::from_iter([value]);

    println!("Adding Property");

    // Add the Property to the federation
    hierarchies_client
        .add_property(
            federation_id,
            FederationProperty::new(property_name.clone()).with_allowed_values(allowed_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to add Property")?;

    println!("Added Property");

    // A receiver is an account that will receive the attestation
    let receiver = ObjectID::random();

    // Properties
    let properties = FederationProperty {
        name: property_name.clone(),
        allowed_values,
        shape: None,
        allow_any: false,
        timespan: Timespan::default(),
    };

    // Let us issue an accreditation to attest to the Property
    hierarchies_client
        .create_accreditation_to_attest(federation_id, receiver, vec![properties])
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to issue accreditation to attest")?;

    println!("Issued accreditation to attest");

    // Check if the accreditation was issued
    let federation: Federation = hierarchies_client.get_federation_by_id(federation_id).await?;

    println!("Federation: {federation:#?}");

    // Check if the receiver has the accreditation to attest
    let properties = federation.governance.accreditations_to_attest.contains_key(&receiver);

    assert!(properties);

    Ok(())
}
