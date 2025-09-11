// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation } from "@iota/hierarchies/node";
import assert from "assert";
import { generateRandomAddress, getFundedClient } from "./util";

export async function reinstateRootAuthority(): Promise<void> {
    const hierarchies = await getFundedClient();

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies
        .createNewFederation()
        .buildAndExecute(hierarchies);

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    // Add a second root authority first
    const secondRootAuthority: string = generateRandomAddress();
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

    // Get the updated federation after revocation
    updatedFederation = await hierarchies.readOnly().getFederationById(federation.id);
    console.log("Total root authorities after revocation: ", updatedFederation.rootAuthorities.length);
    console.log("Revoked root authorities count: ", updatedFederation.revokedRootAuthorities.length);

    // Now reinstate the revoked root authority
    console.log("\n🔄 Reinstating second root authority: ", secondRootAuthority);
    await hierarchies
        .reinstateRootAuthority(federation.id, secondRootAuthority)
        .buildAndExecute(hierarchies);

    console.log("\n✅ Root authority reinstated successfully!");

    // Verify the root authority was reinstated
    const isReinstatedRootAuthority = await hierarchies.readOnly().isRootAuthority(federation.id, secondRootAuthority);
    console.log("Is second authority a root authority after reinstatement: ", isReinstatedRootAuthority);
    assert(isReinstatedRootAuthority, "Second root authority should be active after reinstatement");

    // Get the final federation state
    const finalFederation: Federation = await hierarchies.readOnly().getFederationById(federation.id);
    console.log("\nFinal federation state:");
    console.log("Total root authorities after reinstatement: ", finalFederation.rootAuthorities.length);
    console.log("Revoked root authorities count: ", finalFederation.revokedRootAuthorities.length);

    // Print all active root authorities
    console.log("Active root authorities:");
    finalFederation.rootAuthorities.forEach((ra, index) => {
        console.log(`  - Root Authority ${index + 1}: ${ra.accountId}`);
    });

    // Print revoked root authorities (should be empty now)
    console.log("Revoked root authorities:");
    if (finalFederation.revokedRootAuthorities.length === 0) {
        console.log("  - None (all previously revoked authorities have been reinstated)");
    } else {
        finalFederation.revokedRootAuthorities.forEach((ra, index) => {
            console.log(`  - Revoked Authority ${index + 1}: ${ra}`);
        });
    }
}
