// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation } from "@iota/hierarchies/node";
import { getFundedClient } from "./util";


export async function addRootAuthority(): Promise<void> {
    const hierarchiesClient = await getFundedClient();

    // Create a new federation
    const { output: federation } : { output : Federation} = (await hierarchiesClient
        .createNewFederation()
        .buildAndExecute(hierarchiesClient));

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    const newRootAuthority : string = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    // Add the root authority to the federation
    await hierarchiesClient
        .addRootAuthority(federation.id, newRootAuthority)
        .buildAndExecute(hierarchiesClient);

    console.log("\n✅ Root authority added successfully!");

    // Get the updated federation and print it
    const updatedFederation: Federation = await hierarchiesClient.readOnly().getFederationById(federation.id);

    // Check if the root authority was added
    const rootAuthorities = updatedFederation.rootAuthorities;
    const wasAdded = rootAuthorities.some(ra => ra.accountId === newRootAuthority);

    if (wasAdded) {
        console.log("\n✅ Root authority was successfully added to the federation.");
    } else {
        console.error("\n❌ Root authority was not added to the federation.");
    }
}
