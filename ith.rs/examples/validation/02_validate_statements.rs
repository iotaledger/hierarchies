// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use anyhow::Context;
use iota_sdk::types::base_types::ObjectID;
use ith::core::types::statements::name::StatementName;
use ith::core::types::statements::value::StatementValue;
use ith::core::types::statements::Statement;
use ith::core::types::timespan::Timespan;
use trust_hierarchies_examples::get_funded_client;

/// Demonstrates how to use the offchain API to validate statements.
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.md

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ith_client = get_funded_client().await?;

    let federation = ith_client
        .create_new_federation()
        .build_and_execute(&ith_client)
        .await?;
    let federation_id = federation.output.id.object_id();

    //   Add Statement
    let statement_name = StatementName::from("Example LTD");
    let value = StatementValue::Text("Hello".to_owned());
    let allowed_values = HashSet::from_iter([value.clone()]);

    ith_client
        .add_statement(*federation_id, statement_name.clone(), allowed_values.clone(), false)
        .build_and_execute(&ith_client)
        .await
        .context("Failed to add Statement")?;

    // Add new receiver
    let receiver = ObjectID::random();

    // Statements
    let statements = Statement {
        statement_name: statement_name.clone(),
        allowed_values,
        condition: None,
        allow_any: false,
        timespan: Timespan::default(),
    };

    // Let us issue a permission to attest to the Statement
    {
        ith_client
            .create_accreditation_to_attest(*federation_id, receiver, vec![statements.clone()])
            .build_and_execute(&ith_client)
            .await
            .context("Failed to issue permission to attest")?;
    }

    // Validate statements
    let statements = [(statement_name, value)];

    let validate = ith_client
        .validate_statements(*federation_id, (*receiver).into(), statements)
        .await;

    assert!(validate.is_ok());

    println!("Validated statements");

    Ok(())
}
