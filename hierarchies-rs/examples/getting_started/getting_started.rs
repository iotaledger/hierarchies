// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use anyhow::Context;
use hierarchies::core::types::property::FederationProperty;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::property_value::PropertyValue;
use hierarchies_examples::get_funded_client;
use iota_sdk::types::base_types::ObjectID;

/// Getting started
///
/// When the Hierarchies package is published it creates a new federation,
/// adds Properties, creates an attestation, validates them,
/// Before running the example:
///  - ensure you have the IOTA CLI installed and configured for the the selected network
///  - IOTA_HIERARCHIES_PKG_ID env is set to the Hierarchies package ID
///  - API_URL env is set to the IOTA node URL
///
/// Please note that we use an unsecured private key provider [`TestMemSigner`],
/// which should NOT be used in production.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = get_funded_client().await?;

    // Create a Property with allowed values
    let property_name = PropertyName::new(["university", "a", "score", "department"]);
    let value_biology = PropertyValue::Text("biology".to_owned());
    let value_physics = PropertyValue::Text("physics".to_owned());

    // Allowed values for the property in whole federation
    let allowed_values_properties = HashSet::from([value_biology.clone(), value_physics.clone()]);

    // Create new federation
    println!("Creating a new federation");
    let federation = client.create_new_federation().build_and_execute(&client).await?;
    println!("Federation created");
    let federation_id = *federation.output.id.object_id();

    println!("Adding trusted properties");
    // Add the Property to the federation. The federation owner can add Properties by default.
    client
        .add_property(federation_id, property_name.clone(), allowed_values_properties, false)
        .build_and_execute(&client)
        .await
        .context("Failed to add a Property")?;
    println!("✅ Added Property");

    // Lets delegate the trust to another account and create an accreditation withe the Statement
    // The receiver account will be able to attest to the Property `university.a.score.department`
    // and value `physics` on behalf of the Federation

    // An attester is an account that will receive the accreditation to attest
    let attester = ObjectID::random();
    // Allowed values for the attestation
    let allowed_values_attestation = [value_physics.clone()];

    // Properties
    let properties = FederationProperty::new(property_name.clone()).with_allowed_values(allowed_values_attestation);

    // Create an accreditation to attest to the Statement
    client
        .create_accreditation_to_attest(federation_id, attester, [properties])
        .build_and_execute(&client)
        .await
        .context("Failed creating attestation")?;
    println!("✅ Accreditation to attest has been created for the user {attester}");

    // Let's validate the Properties. Validation is a process of checking if the accreditation
    // receiver is accredited to attest to the Property with the given Property Value
    client
        .validate_properties(
            federation_id,
            attester,
            [(property_name.clone(), value_physics.clone())],
        )
        .await
        .context("Failed to validate Properties")?;
    println!("✅ Validated Properties");

    // // TODO replace with revoke_accreditation_to_attest
    // client
    //     .revoke_accreditation_to_attest(federation_id, attester, 0)
    //     .build_and_execute(&client)
    //     .await
    //     .context("Failed to revoke attestation")?;

    // println!("✅ Revoked attestation");

    // // Validate Properties again - it should returned an error
    // let expected_error = client
    //     .validate_properties(
    //         federation_id,
    //         attester,
    //         [(property_name.clone(), value_physics.clone())],
    //     )
    //     .await;
    // assert!(expected_error.is_err());
    // println!("✅ Expected error on validation after revocation for '{value_physics:?}'");

    // println!("🎉 Done");
    Ok(())
}
