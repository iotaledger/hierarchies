// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, StatementName, StatementValue } from "@iota/hierarchies/node";
import assert from "assert";
import { getFundedClient } from "./util";

/**
 * Demonstrates how to add a Statement to a federation.
 */
export async function addStatement(): Promise<void> {
    // Get the client instance
    const hierarchies = await getFundedClient();

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies
        .createNewFederation()
        .buildAndExecute(hierarchies);

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    // Trusted property name
    const statementName = new StatementName(["company", "example"]);

    // Trusted property value
    const value = StatementValue.newText("Hello");
    const anotherValue = StatementValue.newText("World");
    const allowedValues = [value, anotherValue];

    // Add the Statement to the federation
    await hierarchies
        .addStatement(federation.id, statementName, allowedValues, false)
        .buildAndExecute(hierarchies);

    console.log(`\n✅ Statement: ${statementName.dotted()} was added successfully.`);

    // Get the updated federation
    const updatedFederation: Federation = await hierarchies.readOnly().getFederationById(federation.id);

    const addedStatement = updatedFederation.governance.statements.data.find(s =>
        s.statementName.dotted() === statementName.dotted()
    );
    assert(addedStatement, `Didn't find the Statement in the Federation: ${statementName.dotted()}`);

    console.log("\n✅ Statement was successfully added to the federation.");
}
