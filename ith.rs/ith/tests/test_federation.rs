use anyhow::Context;
use iota_sdk::types::base_types::ObjectID;
use utils::TestClient;

mod utils;

#[tokio::test]
async fn test_add_root_authority() -> anyhow::Result<()> {
  let client = TestClient::init().await?;

  let ith_client = client.ith_client().await?;

  let federation = ith_client.new_federation(None).await?;

  let id = ObjectID::random();
  ith_client
    .add_root_authority(*federation.id.object_id(), id, None)
    .await
    .context("Failed to add trusted property")?;

  println!("added_authority: {:?}", ());

  let federation = ith_client.offchain(*federation.id.object_id()).await?;

  // Check the account
  assert!(federation
    .federation()
    .root_authorities
    .iter()
    .any(|ra| ra.account_id == id));

  Ok(())
}

#[tokio::test]
#[ignore = "This test is not working"]
async fn test_adding_trusted_properties() -> anyhow::Result<()> {
  let client = TestClient::init().await?;

  let ith_client = client.ith_client().await?;

  let _federation = ith_client.new_federation(None).await?;

  // ith_client
  //   .add_trusted_property(
  //     &client,
  //     TrustedPropertyName {
  //       name: vec!["Home".to_string()],
  //     },
  //     VecSet {
  //       contents: vec![TrustedPropertyValue {
  //         value: "12345".to_string(),
  //       }],
  //     },
  //     true,
  //   )
  //   .await
  //   .context("Failed to add trusted property")?;

  Ok(())
}
