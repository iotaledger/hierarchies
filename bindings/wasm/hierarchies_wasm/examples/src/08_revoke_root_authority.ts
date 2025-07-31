// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation } from "@iota/hierarchies/node";
import assert from "assert";
import { randomBytes } from "crypto";
import { getFundedClient } from "./util";

export async function revokeRootAuthority(): Promise<void> {
    const hierarchies = await getFundedClient();

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies
        .createNewFederation()
        .buildAndExecute(hierarchies);

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    // Add a second root authority first
    const secondRootAuthority: string = "0x" + randomBytes(32).toString("hex");
    console.log("Adding second root authority: ", secondRootAuthority);

    await hierarchies
        .addRootAuthority(federation.id, secondRootAuthority)
        .buildAndExecute(hierarchies);

    console.log("\n✅ Second root authority added successfully!");

    // Check if the second root authority is active
    const isRootAuthority = await hierarchies.readOnly().isRootAuthority(federation.id, secondRootAuthority);
    console.log("Is second authority a root authority: ", isRootAuthority);
    assert(isRootAuthority, "Second root authority should be active");

    // Get the federation to see all root authorities
    let updatedFederation: Federation = await hierarchies.readOnly().getFederationById(federation.id);
    console.log("Total root authorities before revocation: ", updatedFederation.rootAuthorities.length);

    // Now revoke the second root authority
    console.log("Revoking second root authority: ", secondRootAuthority);
    await hierarchies
        .revokeRootAuthority(federation.id, secondRootAuthority)
        .buildAndExecute(hierarchies);

    console.log("\n✅ Root authority revoked successfully!");

    // Verify the root authority was revoked
    try {
        const isStillRootAuthority = await hierarchies.readOnly().isRootAuthority(federation.id, secondRootAuthority);
        if (isStillRootAuthority) {
            console.log("❌ Root authority is still active (unexpected)");
        } else {
            console.log("✅ Root authority is no longer active");
        }
    } catch (error) {
        console.log("✅ Root authority check failed as expected (revoked authority)");
    }

    // Get the updated federation
    updatedFederation = await hierarchies.readOnly().getFederationById(federation.id);
    console.log("Total root authorities after revocation: ", updatedFederation.rootAuthorities.length);
    console.log("Revoked root authorities count: ", updatedFederation.revokedRootAuthorities.length);

    // Print remaining root authorities
    console.log("Remaining root authorities:");
    updatedFederation.rootAuthorities.forEach((ra, index) => {
        console.log(`  - Root Authority ${index + 1}: ${ra.accountId}`);
    });

    // Ensure we cannot revoke the last root authority
    if (updatedFederation.rootAuthorities.length === 1) {
        console.log("\nTesting prevention of last root authority revocation...");
        const lastRootAuthority = updatedFederation.rootAuthorities[0].accountId;

        try {
            await hierarchies
                .revokeRootAuthority(federation.id, lastRootAuthority)
                .buildAndExecute(hierarchies);
            console.log("❌ Should not be able to revoke the last root authority");
        } catch (error: any) {
            if (error.toString().includes("9")) {
                console.log("✅ Correctly prevented revocation of last root authority");
            } else {
                console.log("⚠️  Revocation failed for unexpected reason: ", error.toString());
            }
        }
    }
}