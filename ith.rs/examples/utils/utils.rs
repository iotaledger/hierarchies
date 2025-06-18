// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use iota_sdk::{IotaClientBuilder, IOTA_LOCAL_NETWORK_URL};
use ith::client::{ITHClient, ITHClientReadOnly};
use product_common::test_utils::{request_funds, InMemSigner};

pub async fn get_read_only_client() -> anyhow::Result<ITHClientReadOnly> {
    let api_endpoint = std::env::var("API_ENDPOINT").unwrap_or_else(|_| IOTA_LOCAL_NETWORK_URL.to_string());
    let iota_client = IotaClientBuilder::default()
        .build(&api_endpoint)
        .await
        .map_err(|err| anyhow::anyhow!(format!("failed to connect to network; {}", err)))?;

    let package_id = std::env::var("IOTA_ITH_PKG_ID")
        .map_err(|e| {
            anyhow::anyhow!("env variable IOTA_ITH_PKG_ID must be set in order to run the examples").context(e)
        })
        .and_then(|pkg_str| pkg_str.parse().context("invalid package id"))?;

    ITHClientReadOnly::new_with_pkg_id(iota_client, package_id)
        .await
        .context("failed to create a read-only ITHClient")
}

pub async fn get_funded_client() -> Result<ITHClient<InMemSigner>, anyhow::Error> {
    let signer = InMemSigner::new();
    let sender_address = signer.get_address().await?;

    request_funds(&sender_address).await?;

    let read_only_client = get_read_only_client().await?;
    let ith_client: ITHClient<InMemSigner> = ITHClient::new(read_only_client, signer).await?;

    Ok(ith_client)
}
