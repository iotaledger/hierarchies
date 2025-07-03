// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::Deref;
use std::sync::Arc;

use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::crypto::PublicKey;
use iota_interaction::{IotaClientBuilder, IOTA_LOCAL_NETWORK_URL};
use iota_interaction_rust::IotaClientAdapter;
use ith::client::{ITHClient, ITHClientReadOnly};
use product_common::core_client::{CoreClient, CoreClientReadOnly};
use product_common::network_name::NetworkName;
use product_common::test_utils::{init_product_package, request_funds, InMemSigner};
use tokio::sync::OnceCell;

/// Script file for publishing the package.
pub const PUBLISH_SCRIPT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../ith.move/scripts/publish_ith.sh");

static PACKAGE_ID: OnceCell<ObjectID> = OnceCell::const_new();

pub async fn get_funded_test_client() -> anyhow::Result<TestClient> {
    TestClient::new_with_signer(InMemSigner::new()).await
}

#[derive(Clone)]
pub struct TestClient {
    client: Arc<ITHClient<InMemSigner>>,
}

impl Deref for TestClient {
    type Target = ITHClient<InMemSigner>;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl TestClient {
    pub async fn new_with_signer(signer: InMemSigner) -> anyhow::Result<Self> {
        let active_address = signer.get_address().await?;
        request_funds(&active_address).await?;

        let api_endpoint = std::env::var("API_ENDPOINT").unwrap_or_else(|_| IOTA_LOCAL_NETWORK_URL.to_string());
        let client = IotaClientBuilder::default().build(&api_endpoint).await?;
        let package_id = PACKAGE_ID
            .get_or_try_init(|| init_product_package(&client, None, Some(PUBLISH_SCRIPT_FILE)))
            .await
            .copied()?;

        let ith_client = ITHClientReadOnly::new_with_pkg_id(client, package_id).await?;
        let client = ITHClient::new(ith_client, signer).await?;

        Ok(TestClient {
            client: Arc::new(client),
        })
    }
}

impl CoreClientReadOnly for TestClient {
    fn package_id(&self) -> ObjectID {
        self.client.package_id()
    }

    fn network_name(&self) -> &NetworkName {
        self.client.network_name()
    }

    fn client_adapter(&self) -> &IotaClientAdapter {
        self.client.client_adapter()
    }
}

impl CoreClient<InMemSigner> for TestClient {
    fn signer(&self) -> &InMemSigner {
        self.client.signer()
    }

    fn sender_address(&self) -> IotaAddress {
        self.client.sender_address()
    }

    fn sender_public_key(&self) -> &PublicKey {
        self.client.sender_public_key()
    }
}
