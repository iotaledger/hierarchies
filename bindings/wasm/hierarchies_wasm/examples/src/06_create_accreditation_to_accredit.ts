// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    Federation,
    Statement,
    StatementName,
    StatementValue,
} from "@iota/hierarchies/node";
import { getFundedClient } from "./util";
import { HierarchiesClient } from "@iota/hierarchies/node";
import { randomBytes } from "crypto";
import assert from "assert";

/**
 * Demonstrate how to issue a permission to accredit to a Statement.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function createAccreditationToAccredit(client?: HierarchiesClient) {
    console.log("\nRunning create accreditation to accredit example");

    // Get the client instance
    const hierarchies = client ?? (await getFundedClient());

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(hierarchies);
    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    // The name of the statement
    const statementName = new StatementName(["Example LTD"]);

    // The value of the statement
    const value = StatementValue.newText("Hello");

    const allowedValues = [value];

    // Add the statement to the federation
    await hierarchies
        .addStatement(federation.id, statementName, allowedValues, false)
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Statement ${statementName.dotted()} added successfully`);


    // A receiver is an account that will receive the accreditation
    const receiver = "0x" + randomBytes(32).toString("hex");

    // Statements
    const statement = new Statement(statementName).withAllowedValues([StatementValue.newText("Hello")]);

    // Let us issue a permission to accredit to the Statement
    await hierarchies
        .createAccreditationToAccredit(federation.id, receiver, [statement])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to accredit issued successfully for ${receiver}`);


    // Check if the permission was issued
    const accreditationsToAccredit = await hierarchies.readOnly().getAccreditationsToAccredit(federation.id, receiver);

    assert(accreditationsToAccredit.accreditations.length > 0, "Accreditation not found for receiver");
    assert(accreditationsToAccredit.accreditations[0].statements[0].statementName.dotted() === statementName.dotted(), "Statement name does not match");

    console.log("\n✅ Accreditation to accredit found for receiver");
}
