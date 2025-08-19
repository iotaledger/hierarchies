// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use hierarchies::client::{HierarchiesClient, HierarchiesClientReadOnly};
use iota_sdk::{IOTA_LOCAL_NETWORK_URL, IotaClientBuilder};
use product_common::test_utils::{InMemSigner, request_funds};

pub async fn get_read_only_client() -> anyhow::Result<HierarchiesClientReadOnly> {
    let api_endpoint = std::env::var("API_ENDPOINT").unwrap_or_else(|_| IOTA_LOCAL_NETWORK_URL.to_string());
    let iota_client = IotaClientBuilder::default()
        .build(&api_endpoint)
        .await
        .map_err(|err| anyhow::anyhow!(format!("failed to connect to network; {}", err)))?;

    let package_id = std::env::var("IOTA_HIERARCHIES_PKG_ID")
        .map_err(|e| {
            anyhow::anyhow!("env variable IOTA_HIERARCHIES_PKG_ID must be set in order to run the examples").context(e)
        })
        .and_then(|pkg_str| pkg_str.parse().context("invalid package id"))?;

    HierarchiesClientReadOnly::new_with_pkg_id(iota_client, package_id)
        .await
        .context("failed to create a read-only HierarchiesClient")
}

pub async fn get_funded_client() -> Result<HierarchiesClient<InMemSigner>, anyhow::Error> {
    let signer = InMemSigner::new();
    let sender_address = signer.get_address().await?;

    request_funds(&sender_address).await?;

    let read_only_client = get_read_only_client().await?;
    let hierarchies_client: HierarchiesClient<InMemSigner> = HierarchiesClient::new(read_only_client, signer).await?;

    Ok(hierarchies_client)
}
