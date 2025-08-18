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
use product_common::core_client::CoreClient;

/// Demonstrate how to issue an accreditation to accredit to a Property.
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

    // Trusted property name
    let property_name = PropertyName::from("Example LTD");

    // Trusted property value
    let value = PropertyValue::Text("Hello".to_owned());

    let allowed_values = HashSet::from_iter([value]);

    println!("Adding Property");

    // Add the Property to the federation
    hierarchies_client
        .add_property(federation_id, property_name.clone(), allowed_values.clone(), false)
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to add Property")?;

    println!("Added Property");

    // A receiver is an account that will receive the accreditation
    let receiver = ObjectID::random();

    // Properties
    let properties = FederationProperty {
        name: property_name.clone(),
        allowed_values,
        shape: None,
        allow_any: false,
        timespan: Timespan::default(),
    };

    // Let us issue an accreditation to accredit to the Property
    hierarchies_client
        .create_accreditation_to_accredit(federation_id, receiver, vec![properties.clone()])
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to issue accreditation to accredit")?;

    // Issue an accreditation to the original account
    hierarchies_client
        .create_accreditation_to_accredit(
            federation_id,
            hierarchies_client.sender_address().into(),
            vec![properties],
        )
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to issue permission to attest")?;

    println!("Issued accreditation to accredit");

    println!("Checking if the receiver has the accreditation to accredit");
    // Check if the receiver has the permission to accredit
    let can_accredit = hierarchies_client.is_accreditor(federation_id, receiver).await?;
    assert!(can_accredit);

    // Revoke the accreditation
    let accreditations = hierarchies_client
        .get_accreditations_to_accredit(federation_id, receiver)
        .await
        .context("Failed to find accreditation to accredit")?;

    let accreditation_id = accreditations.accreditations[0].id.object_id();

    hierarchies_client
        .revoke_accreditation_to_accredit(federation_id, receiver, *accreditation_id)
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to revoke accreditation to accredit")?;

    // Check if the accreditation was revoked
    let federation: Federation = hierarchies_client.get_federation_by_id(federation_id).await?;

    println!("Federation: {federation:#?}");

    // Check if the receiver has the accreditation to accredit
    let can_accredit = federation.governance.accreditations_to_accredit.get(&receiver).unwrap();

    assert!(can_accredit.accreditations.is_empty());
    Ok(())
}
