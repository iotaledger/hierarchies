use crate::client::get_funded_test_client;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::object::Object;
use ith::core::types::{Event, Federation, FederationCreatedEvent};
use product_common::core_client::{CoreClient, CoreClientReadOnly};

#[tokio::test]
async fn test_creation_of_federation() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    let federation = client.create_new_federation().build_and_execute(&client).await;

    assert!(federation.is_ok());

    let federation = federation.unwrap();
    let tx_response = federation.response;
    let federation = federation.output;

    // we assert that the federation has no root authorities
    assert!(!federation.root_authorities.is_empty());
    //we have one root authority
    assert_eq!(federation.root_authorities.len(), 1);
    assert_eq!(
        federation.root_authorities.first().unwrap().account_id.to_string(),
        client.signer().get_address().await?.to_string()
    );

    assert_eq!(federation.governance.accreditations_to_accredit.len(), 1);
    assert_eq!(federation.governance.accreditations_to_attest.len(), 1);

    let events = tx_response.events.unwrap();
    let event = events.data.first().unwrap();

    let event: Event<FederationCreatedEvent> = bcs::from_bytes(event.bcs.bytes()).unwrap();

    assert_eq!(event.data.federation_address, *federation.id.object_id());

    Ok(())
}

#[tokio::test]
async fn test_creation_of_federation_with_root_authorities() -> anyhow::Result<()> {
    let client = get_funded_test_client().await?;

    let federation = client
        .create_new_federation()
        .build_and_execute(&client)
        .await
        .unwrap()
        .output
        .id;

    let root_authority_id = ObjectID::random();

    client
        .add_root_authority(*federation.object_id(), root_authority_id)
        .build_and_execute(&client)
        .await?;

    // we assert that the federation has one root authority
    let federation: Federation = client.get_object_by_id(*federation.object_id()).await?;
    assert_eq!(federation.root_authorities.len(), 2);
    assert!(federation
        .root_authorities
        .iter()
        .any(|ra| *ra.id.object_id() == root_authority_id));

    Ok(())
}
