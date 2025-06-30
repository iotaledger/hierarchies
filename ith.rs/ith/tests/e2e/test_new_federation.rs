use crate::client::get_funded_test_client;
use ith::core::types::{Event, FederationCreatedEvent};
use product_common::core_client::CoreClient;

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
