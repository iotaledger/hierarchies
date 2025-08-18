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
use product_common::core_client::CoreClient;

/// Demonstrates how to use the offchain API to get accreditations to attest and accredit.
///
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

    let user_id = hierarchies_client.sender_address().into();

    let accreditations = hierarchies_client
        .get_accreditations_to_attest(*federation_id, user_id)
        .await?;

    println!("Accreditations to attest: {accreditations:#?}");

    //  Add Property
    let property_name = PropertyName::from("Example LTD");
    let value = PropertyValue::Text("Hello".to_owned());
    let allowed_values = HashSet::from_iter([value]);

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

    // Let us issue a accreditation to attest to the Property
    {
        hierarchies_client
            .create_accreditation_to_attest(*federation_id, receiver, vec![properties.clone()])
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to issue accreditation to attest")?;
    }

    // Check if the accreditation to attest was issued
    let accreditations = hierarchies_client
        .get_accreditations_to_attest(*federation_id, receiver)
        .await
        .context("Failed to find accreditation to attest")?;

    assert!(accreditations.accreditations.len() == 1);

    println!("Accreditations to attest: {accreditations:#?}");

    // Issue accreditation to accredit
    {
        hierarchies_client
            .create_accreditation_to_accredit(*federation_id, receiver, vec![properties])
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to issue accreditation to accredit")?;
    }

    // Check if the accreditation to accredit was issued
    let accreditations = hierarchies_client
        .get_accreditations_to_accredit(*federation_id, receiver)
        .await
        .context("Failed to find accreditation to accredit")?;

    assert!(accreditations.accreditations.len() == 1);

    Ok(())
}
