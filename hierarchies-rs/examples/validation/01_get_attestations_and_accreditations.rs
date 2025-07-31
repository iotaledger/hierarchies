// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use anyhow::Context;
use hierarchies::core::types::statements::name::StatementName;
use hierarchies::core::types::statements::value::StatementValue;
use hierarchies::core::types::statements::Statement;
use hierarchies::core::types::timespan::Timespan;
use hierarchies_examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;
use product_common::core_client::CoreClient;

/// Demonstrates how to use the offchain API to check if a user has a permission to attest and accredit.
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

    let permissions = hierarchies_client
        .get_accreditations_to_attest(*federation_id, user_id)
        .await?;

    println!("Accreditations: {permissions:#?}");

    //   Add Statement
    let statement_name = StatementName::from("Example LTD");
    let value = StatementValue::Text("Hello".to_owned());
    let allowed_values = HashSet::from_iter([value]);

    hierarchies_client
        .add_statement(*federation_id, statement_name.clone(), allowed_values.clone(), false)
        .build_and_execute(&hierarchies_client)
        .await
        .context("Failed to add Statement")?;

    // Add new receiver
    let receiver = ObjectID::random();

    // Statements
    let statements = Statement {
        statement_name,
        allowed_values,
        condition: None,
        allow_any: false,
        timespan: Timespan::default(),
    };

    // Let us issue a permission to attest to the Statement
    {
        hierarchies_client
            .create_accreditation_to_attest(*federation_id, receiver, vec![statements.clone()])
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to issue permission to attest")?;
    }

    // Check if the permission was issued
    let permissions = hierarchies_client
        .get_accreditations_to_attest(*federation_id, receiver)
        .await
        .context("Failed to find permission to attest")?;

    assert!(permissions.statements.len() == 1);

    println!("Accreditations: {permissions:#?}");

    // Issue Accredit permission
    {
        hierarchies_client
            .create_accreditation_to_accredit(*federation_id, receiver, vec![statements])
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to issue permission to accredit")?;
    }

    // Check if the permission was issued
    let permissions = hierarchies_client
        .get_accreditations_to_accredit(*federation_id, receiver)
        .await
        .context("Failed to find permission to accredit")?;

    assert!(permissions.statements.len() == 1);

    Ok(())
}
