// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation } from "@iota/hierarchies/node";
import { IotaClient } from "@iota/iota-sdk/client";
import { strict as assert } from "assert";
import { generateRandomAddress, getFundedClient, NETWORK_URL } from "./util";

export async function addRootAuthority(): Promise<void> {
    const hierarchies = await getFundedClient();

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies
        .createNewFederation()
        .buildAndExecute(hierarchies);

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);
    console.log("Package ID: ", hierarchies.packageId());
    console.log("Package History: ", hierarchies.packageHistory());
    console.log("Sender Address: ", hierarchies.senderAddress());

    // Debug: Query owned objects to see what capabilities we have
    const iotaClient = new IotaClient({ url: NETWORK_URL });
    const senderAddress = hierarchies.senderAddress();
    const structType = `${hierarchies.packageId()}::main::RootAuthorityCap`;
    console.log("Looking for struct type: ", structType);

    const ownedObjects = await iotaClient.getOwnedObjects({
        owner: senderAddress,
        filter: { StructType: structType },
        options: { showType: true, showContent: true },
    });
    console.log("Owned RootAuthorityCap objects:", JSON.stringify(ownedObjects, null, 2));

    // Query ALL owned objects (not filtered)
    const allOwnedObjects = await iotaClient.getOwnedObjects({
        owner: senderAddress,
        options: { showType: true },
    });
    console.log("All owned objects:");
    for (const obj of allOwnedObjects.data) {
        console.log("  - ", obj.data?.type);
    }

    const newRootAuthority: string = generateRandomAddress();

    // Add the root authority to the federation with retry logic
    // This is necessary on testnet due to indexer latency
    let retries = 5;
    let lastError: unknown;
    while (retries > 0) {
        try {
            console.log(`\n⏳ Attempting to add root authority (${6 - retries}/5)...`);
            await hierarchies
                .addRootAuthority(federation.id, newRootAuthority)
                .buildAndExecute(hierarchies);
            break;
        } catch (error) {
            lastError = error;
            retries--;
            if (retries > 0) {
                console.log("⏳ Waiting for indexer to catch up (3s)...");
                await new Promise((resolve) => setTimeout(resolve, 3000));
            }
        }
    }
    if (retries === 0) {
        throw lastError;
    }

    console.log("\n✅ Root authority added successfully!");

    // Get the updated federation
    const updatedFederation: Federation = await hierarchies.readOnly().getFederationById(federation.id);

    // Check if the root authority was added
    assert(
        updatedFederation.rootAuthorities.some(ra => ra.accountId === newRootAuthority),
        "Root authority was not added to the federation.",
    );
    console.log("\n✅ Root authority was successfully added to the federation.");
}
