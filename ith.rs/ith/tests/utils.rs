// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use fastcrypto::traits::{KeyPair, ToFromBytes};
use iota::client_commands;
use iota_keys::keystore::{AccountKeystore, InMemKeystore};
use iota_sdk::types::base_types::{IotaAddress, ObjectID};
use iota_sdk::types::crypto::{IotaSignature, SignatureScheme};
use iota_sdk::{IotaClient, IotaClientBuilder};
use ith::client::{ITHClient, ITHClientReadOnly};
use ith::key::IotaKeySignature;
use jsonpath_rust::JsonPathQuery;
use secret_storage::{SignatureScheme as SignerSignatureScheme, Signer as SignerTrait};
use serde_json::Value;
use tokio::process::Command;

const SCRIPT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts");

pub const TEST_ALIAS: &str = "strange-prase";
pub const GAS_LOCAL_NETWORK: &str = "http://127.0.0.1:9123/gas";

const CACHED_PKG_ID: &str = "../target/ith_pkg_id.txt";

#[derive(Clone)]
pub struct TestClient {
  client: IotaClient,
  package_id: ObjectID,
  signer: TestMemSigner,
}

impl TestClient {
  pub async fn init() -> anyhow::Result<TestClient> {
    let client = IotaClientBuilder::default().build_localnet().await?;

    let signer = TestMemSigner::new();

    let deployer_address = Self::active_address().await?;

    let address = signer.get_address()?;

    client_commands::request_tokens_from_faucet(
      deployer_address.to_owned(),
      GAS_LOCAL_NETWORK.to_owned(),
    )
    .await
    .context("Failed to request tokens from faucet")?;

    client_commands::request_tokens_from_faucet(address.to_owned(), GAS_LOCAL_NETWORK.to_owned())
      .await
      .context("Failed to request tokens from faucet")?;

    // Sleep 1 second
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let package_id =
      if let Ok(id) = std::env::var("ITH_PKG_ID").or(get_cached_id(deployer_address).await) {
        std::env::set_var("ITH_PKG_ID", id.clone());
        id.parse()?
      } else {
        publish_package(deployer_address).await?
      };

    Ok(TestClient {
      client,
      package_id,
      signer,
    })
  }

  pub async fn ith_client(&self) -> anyhow::Result<ITHClient<TestMemSigner>> {
    let read_only_client = ITHClientReadOnly::new(self.client.clone(), self.package_id);

    ITHClient::new(read_only_client, self.signer.clone()).await
  }

  async fn active_address() -> anyhow::Result<IotaAddress> {
    Command::new("iota")
      .arg("client")
      .arg("active-address")
      .arg("--json")
      .output()
      .await
      .context("Failed to execute command")
      .and_then(|output| Ok(serde_json::from_slice::<IotaAddress>(&output.stdout)?))
  }
}

impl Deref for TestClient {
  type Target = IotaClient;
  fn deref(&self) -> &Self::Target {
    &self.client
  }
}

async fn get_cached_id(active_address: IotaAddress) -> anyhow::Result<String> {
  let cache = tokio::fs::read_to_string(CACHED_PKG_ID).await?;
  let (cached_id, cached_address) = cache
    .split_once(';')
    .ok_or(anyhow!("Invalid or empty cached data"))?;

  if cached_address == active_address.to_string().as_str() {
    Ok(cached_id.to_owned())
  } else {
    Err(anyhow!("A network change has invalidated the cached data"))
  }
}

async fn publish_package(active_address: IotaAddress) -> anyhow::Result<ObjectID> {
  let output = Command::new("sh")
    .current_dir(SCRIPT_DIR)
    .arg("publish_ith.sh")
    .output()
    .await?;

  if !output.status.success() {
    anyhow::bail!(
      "Failed to publish move package: \n\n{}\n\n{}",
      std::str::from_utf8(&output.stdout).unwrap(),
      std::str::from_utf8(&output.stderr).unwrap()
    );
  }

  let publish_result = {
    let output_str = std::str::from_utf8(&output.stdout).unwrap();
    let start_of_json = output_str.find('{').ok_or(anyhow!("No json in output"))?;
    serde_json::from_str::<Value>(output_str[start_of_json..].trim())?
  };

  let package_id = publish_result
    .path("$.objectChanges[?(@.type == 'published')].packageId")
    .map_err(|e| anyhow!("Failed to parse JSONPath: {e}"))
    .and_then(|value| Ok(serde_json::from_value::<Vec<ObjectID>>(value)?))?
    .first()
    .copied()
    .ok_or_else(|| anyhow!("Failed to parse package ID after publishing"))?;

  // Persist package ID in order to avoid publishing the package for every test.
  let package_id_str = package_id.to_string();
  std::env::set_var("ITH_PKG_ID", package_id_str.as_str());
  let mut file = std::fs::File::create(CACHED_PKG_ID)?;
  write!(&mut file, "{};{}", package_id_str, active_address)?;

  Ok(package_id)
}

#[derive(Clone)]
pub struct TestMemSigner(pub Arc<InMemKeystore>);

impl TestMemSigner {
  pub fn new() -> Self {
    let mut mem = InMemKeystore::new_insecure_for_tests(0);
    mem
      .generate_and_add_new_key(
        SignatureScheme::ED25519,
        Some(TEST_ALIAS.to_owned()).to_owned(),
        None,
        None,
      )
      .expect("Could not generate key");

    TestMemSigner(Arc::new(mem))
  }

  pub fn get_address(&self) -> anyhow::Result<IotaAddress> {
    let address = self.0.get_address_by_alias(TEST_ALIAS.to_owned())?;
    Ok(*address)
  }
}

impl Default for TestMemSigner {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait::async_trait]
impl SignerTrait<IotaKeySignature> for TestMemSigner {
  type KeyId = ();
  async fn sign(
    &self,
    hash: &[u8],
  ) -> secret_storage::Result<<IotaKeySignature as SignerSignatureScheme>::Signature> {
    let address = self.0.get_address_by_alias(TEST_ALIAS.to_owned()).unwrap();

    let signature = self.0.sign_hashed(address, hash).unwrap();

    Ok(signature.signature_bytes().to_vec())
  }

  async fn public_key(
    &self,
  ) -> secret_storage::Result<<IotaKeySignature as secret_storage::SignatureScheme>::PublicKey> {
    let address = self.0.get_address_by_alias(TEST_ALIAS.to_owned()).unwrap();
    let res = self.0.get_key(address).unwrap();

    let public_key = match res {
      iota_sdk::types::crypto::IotaKeyPair::Ed25519(key) => key.public().as_bytes().to_vec(),
      _ => panic!(),
    };

    Ok(public_key)
  }
  fn key_id(&self) -> &Self::KeyId {
    unimplemented!()
  }
}
