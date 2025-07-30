// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, StatementName, StatementValue, Statement } from "@iota/hierarchies/node";
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

    // Federation ID
    const federationId = federation.id;

    // Trusted property name
    const statementName = new StatementName(["company", "example"]);
    const statementNameDotted = statementName.dotted();

    // Trusted property value
    const value = StatementValue.fromText("Hello");
    const anotherValue = StatementValue.fromText("World");
    const allowedValues = [value, anotherValue];


    // Add the Statement to the federation
    await hierarchies
        .addStatement(federationId, statementName, allowedValues, false)
        .buildAndExecute(hierarchies);

    // Get the updated federation
    const updatedFederation: Federation = await hierarchies.readOnly().getFederationById(federationId);


    for (const [i,  statement] of updatedFederation.governance.statements.data.entries()) {
        if (statement.statementName.dotted() == statementNameDotted) {
            console.log(`\n✅ Statement: ${statement.statementName.dotted()} was added successfully.`);
            return
        }
    }

    console.error("\n❌ Didn't find the Statement in the Federation: ", statementNameDotted);
}
