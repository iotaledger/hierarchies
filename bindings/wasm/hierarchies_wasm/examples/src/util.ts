// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Ed25519KeypairSigner } from "@iota/iota-interaction-ts/node/test_utils";
import { IotaClient } from "@iota/iota-sdk/client";
import { getFaucetHost, requestIotaFromFaucetV0 } from "@iota/iota-sdk/faucet";
import { Ed25519Keypair } from "@iota/iota-sdk/keypairs/ed25519";
import { HierarchiesClient, HierarchiesClientReadOnly } from "@iota/hierarchies/node";

export const IOTA_HIERARCHIES_PKG_ID = globalThis?.process?.env?.IOTA_HIERARCHIES_PKG_ID || "0x17108a3c13a5c919acf4857fb4d6bfe7fa5494cf8ab0cd18c42e6aa09d2a5283";
export const NETWORK_NAME_FAUCET = globalThis?.process?.env?.NETWORK_NAME_FAUCET || "localnet";
export const NETWORK_URL = globalThis?.process?.env?.NETWORK_URL || "http://127.0.0.1:9000";

if (!IOTA_HIERARCHIES_PKG_ID) {
    throw new Error("IOTA_HIERARCHIES_PKG_ID env variable must be set to run the examples");
}
export const TEST_GAS_BUDGET = BigInt(50_000_000);

export async function requestFunds(address: string) {
    await requestIotaFromFaucetV0({
        host: getFaucetHost(NETWORK_NAME_FAUCET),
        recipient: address,
    });
}

export async function getFundedClient(): Promise<HierarchiesClient> {
    if (!IOTA_HIERARCHIES_PKG_ID) {
        throw new Error(`IOTA_HIERARCHIES_PKG_ID env variable must be provided to run the examples`);
    }

    const iotaClient = new IotaClient({ url: NETWORK_URL });

    const hierarchiesClientReadOnly = await HierarchiesClientReadOnly.createWithPkgId(
        iotaClient,
        IOTA_HIERARCHIES_PKG_ID,
    );

    // generate new key
    const keypair = Ed25519Keypair.generate();

    // create signer
    let signer = new Ed25519KeypairSigner(keypair);
    const hierarchiesClient = await new HierarchiesClient(hierarchiesClientReadOnly, signer);

    await requestFunds(hierarchiesClient.senderAddress());

    const balance = await iotaClient.getBalance({ owner: hierarchiesClient.senderAddress() });
    if (balance.totalBalance === "0") {
        throw new Error("Balance is still 0");
    } else {
        console.log(
            `Received gas from faucet: ${balance.totalBalance} for owner ${hierarchiesClient.senderAddress()}`,
        );
    }

    return hierarchiesClient;
}
