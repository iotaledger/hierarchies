// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation } from "@iota/hierarchies/node";
import assert from "assert";
import { randomBytes } from "crypto";
import { getFundedClient } from "./util";

export async function addRootAuthority(): Promise<void> {
    const hierarchies = await getFundedClient();

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies
        .createNewFederation()
        .buildAndExecute(hierarchies);

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    const newRootAuthority: string = "0x" + randomBytes(32).toString("hex");

    // Add the root authority to the federation
    await hierarchies
        .addRootAuthority(federation.id, newRootAuthority)
        .buildAndExecute(hierarchies);

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
