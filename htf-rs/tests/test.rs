// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use iota_sdk::types::base_types::IotaAddress;
use iota_sdk::IotaClient;
use iota_sdk::IotaClientBuilder;
use tokio::process::Command;

pub const LOCAL_NETWORK: &str = "http://127.0.0.1:9000";

pub async fn get_client(network: &str) -> anyhow::Result<IotaClient> {
    let client = IotaClientBuilder::default().build(network).await?;

    Ok(client)
}

pub async fn request_funds(address: &IotaAddress) -> anyhow::Result<()> {
    let output = Command::new("iota")
        .arg("client")
        .arg("faucet")
        .arg("--address")
        .arg(address.to_string())
        .arg("--json")
        .output()
        .await
        .context("Failed to execute command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to request funds from faucet: {}",
            std::str::from_utf8(&output.stderr).unwrap()
        );
    }

    Ok(())
}
