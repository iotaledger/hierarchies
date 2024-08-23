use anyhow::Context;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::id::ID;
use utils::TestClient;

mod utils;

#[tokio::test]
async fn test_add_root_authority() -> anyhow::Result<()> {
  let client = TestClient::init().await?;

  let htf_client = client.htf_client().await?;

  let federation = htf_client.new_federation().await?;

  let id = ID::new(ObjectID::random());
  htf_client
    .add_root_authority(federation, id.clone())
    .await
    .context("Failed to add trusted property")?;

  println!("added_authority: {:?}", ());

  let federation = htf_client.offchain(federation).await?;

  // Check the account
  assert!(federation
    .federation()
    .root_authorities
    .iter()
    .any(|ra| ra.account_id == id));

  Ok(())
}

#[tokio::test]
async fn test_adding_trusted_properties() -> anyhow::Result<()> {
  // let client = TestClient::init().await?;

  // let federation = client.htf_client().await?.new_federation().await?;

  // federation
  //     .add_trusted_property(
  //         &client,
  //         TrustedPropertyName {
  //             name: vec!["Home".to_string()],
  //         },
  //         VecSet {
  //             contents: vec![TrustedPropertyValue {
  //                 value: "12345".to_string(),
  //             }],
  //         },
  //         true,
  //     )
  //     .await
  //     .context("Failed to add trusted property")?;

  Ok(())
}
