// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, FederationProperty, HierarchiesClient, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import { strict as assert } from "assert";
import { generateRandomAddress, getFundedClient } from "./util";

/**
 * Regression test for the federation value-scope gate.
 *
 * A federation property declares a bounded value space (`allowed_values = {"a"}`).
 * Issuing an accreditation for a value outside that space (`"b"`) must be
 * rejected on-chain (EPropertyValuesOutOfFederationScope), even though the
 * caller is a root authority.
 */
export async function createAccreditationOutOfScopeFails(client?: HierarchiesClient) {
    console.log("\nRunning out-of-scope accreditation rejection test");

    const hierarchies = client ?? (await getFundedClient());

    // Create a new federation.
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(
        hierarchies,
    );
    console.log("Federation ID: ", federation.id);

    // Federation declares test-property with allowed_values = {"a"}.
    const propertyName = new PropertyName(["test-property"]);
    await hierarchies
        .addProperty(
            federation.id,
            new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("a")]),
        )
        .buildAndExecute(hierarchies);

    const receiver = generateRandomAddress();

    // Request an accreditation for "b", which is outside the federation's value space.
    const outOfScope = new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("b")]);

    // The transaction must be rejected.
    await assert.rejects(
        hierarchies
            .createAccreditationToAttest(federation.id, receiver, [outOfScope])
            .buildAndExecute(hierarchies),
        "out-of-scope accreditation should be rejected on-chain",
    );

    console.log("\n✅ Out-of-scope accreditation correctly rejected");
}
