use anyhow::Context;

use htf::htf::Federation;
use htf::types::{TrustedPropertyName, TrustedPropertyValue};
use iota_sdk::types::collection_types::VecSet;
use iota_sdk::types::id::ID;

use iota_sdk::types::base_types::ObjectID;

use utils::TestClient;

mod utils;

#[tokio::test]
async fn test_add_root_authority() -> anyhow::Result<()> {
    let client = TestClient::init().await?;

    let (mut federation, client) = client.create_new_federation().await?;

    let id = ID::new(ObjectID::random());
    federation
        .add_root_authority(&client, id.clone())
        .await
        .context("Failed to add trusted property")?;

    println!("added_authority: {:?}", ());

    federation = Federation::get_federation_by_id(federation.id(), &client)
        .await
        .context("Failed to get federation")?;

    // Check the account
    assert!(federation
        .root_authorities
        .iter()
        .any(|ra| ra.account_id == id));

    Ok(())
}

#[tokio::test]
async fn test_adding_trusted_properties() -> anyhow::Result<()> {
    let client = TestClient::init().await?;

    let (federation, client) = client.create_new_federation().await?;

    federation
        .add_trusted_property(
            &client,
            TrustedPropertyName {
                name: "Home".to_string(),
            },
            VecSet {
                contents: vec![TrustedPropertyValue {
                    value: "12345".to_string(),
                }],
            },
            true,
        )
        .await
        .context("Failed to add trusted property")?;

    Ok(())
}
