use anyhow::Context;
use htf::client::HTFClient;
use htf::types::{TrustedPropertyName, TrustedPropertyValue};
use iota::client_commands::request_tokens_from_faucet;
use iota_sdk::types::collection_types::VecSet;
use iota_sdk::types::id::ID;

use fastcrypto::traits::{KeyPair, ToFromBytes};
use htf::htf::Federation;
use htf::key::IotaKeySignature;
use iota_keys::keystore::{AccountKeystore, InMemKeystore};
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::crypto::{IotaSignature, SignatureScheme};
use secret_storage::prelude::KeySignatureTypes;
use secret_storage::signer::Signer;
use tokio::sync::OnceCell;

pub const DEFAULT_ALIAS: &str = "strange-prase";

static CLIENT: OnceCell<TestClient> = OnceCell::const_new();

pub const GAS_LOCAL_NETWORK: &str = "http://127.0.0.1:9123/gas";

pub struct TestClient {
    pub client: iota_sdk::IotaClient,
}

impl TestClient {
    pub async fn new() -> anyhow::Result<Self> {
        let client = iota_sdk::IotaClientBuilder::default()
            .build_localnet()
            .await?;

        Ok(Self { client })
    }
}

pub async fn get_client() -> anyhow::Result<iota_sdk::IotaClient> {
    let client = CLIENT.get_or_try_init(TestClient::new).await?;
    Ok(client.client.clone())
}

pub const TEST_GAS_BUDGET: u64 = 50_000_000;

async fn create_new_federation() -> anyhow::Result<(Federation, HTFClient)> {
    let mut mem = InMemKeystore::new_insecure_for_tests(0);
    mem.generate_and_add_new_key(
        SignatureScheme::ED25519,
        Some(DEFAULT_ALIAS.to_owned()),
        None,
        None,
    )
    .expect("Could not generate key");

    let signer = TestMemSigner(mem);

    let client = get_client().await?;
    let htf_hex_str = "0x1402f0d5ac4675492f5954abfc884ccf317f00d9e7926bda2f00b3c076971ea4";
    let htf_package_id = ObjectID::from_hex_literal(htf_hex_str)?;
    let sender_public_key = default_user_key(&signer).await?;
    let htf_client = htf::client::HTFClientBuilder::default()
        .htf_package_id(htf_package_id)
        .iota_client(client)
        .sender_public_key(&sender_public_key)
        .signer(Box::new(signer))
        .gas_budget(TEST_GAS_BUDGET)
        .build()?;

    let federation = Federation::new(&htf_client).await?;

    Ok((federation, htf_client))
}

#[tokio::test]
async fn test_add_root_authority() -> anyhow::Result<()> {
    let (mut federation, client) = create_new_federation().await?;

    let id = ID::new(ObjectID::random());
    federation
        .add_root_authority(&client, id.clone())
        .await
        .context("Failed to add trusted property")?;

    println!("added_authority: {:?}", ());

    //Update the federation
    federation
        .refresh(&client)
        .await
        .context("Failed to update federation")?;

    // Check the account
    assert!(federation
        .root_authorities
        .iter()
        .any(|ra| ra.account_id == id));

    Ok(())
}

#[tokio::test]
async fn test_adding_trusted_properties() -> anyhow::Result<()> {
    let (federation, client) = create_new_federation().await?;

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

pub async fn default_user_key(storage: &TestMemSigner) -> anyhow::Result<Vec<u8>> {
    let address = storage.0.get_address_by_alias(DEFAULT_ALIAS.to_owned())?;
    println!("Address: {:?}", address);

    request_tokens_from_faucet(address.to_owned(), GAS_LOCAL_NETWORK.to_owned())
        .await
        .context("Failed to request tokens from faucet")?;

    // Sleep 1 second
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let res = storage.0.get_key(address)?;

    let public_key = match res {
        iota_sdk::types::crypto::IotaKeyPair::Ed25519(key) => key.public().as_bytes().to_vec(),
        _ => panic!(),
    };

    Ok(public_key)
}

pub struct TestMemSigner(InMemKeystore);

#[async_trait::async_trait]
impl Signer<IotaKeySignature> for TestMemSigner {
    async fn sign(
        &self,
        hash: &[u8],
    ) -> Result<<IotaKeySignature as KeySignatureTypes>::Signature, anyhow::Error> {
        let address = self.0.get_address_by_alias(DEFAULT_ALIAS.to_owned())?;

        let signature = self.0.sign_hashed(address, hash)?;

        Ok(signature.signature_bytes().to_vec())
    }
}
