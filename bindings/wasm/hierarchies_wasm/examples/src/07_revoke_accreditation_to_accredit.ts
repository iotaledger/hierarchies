// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, HierarchiesClient, Statement, StatementName, StatementValue } from "@iota/hierarchies/node";
import assert from "assert";
import { randomBytes } from "crypto";
import { getFundedClient } from "./util";

/**
 * Demonstrate how to revoke a permission to accredit to a Statement.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function revokeAccreditationToAccredit(client?: HierarchiesClient) {
    console.log("\nRunning revoke accreditation to accredit example");

    // Get the client instance
    const hierarchies = client ?? (await getFundedClient());

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(
        hierarchies,
    );
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

    // Check if the receiver has the permission to accredit
    let accreditationsToAccredit = await hierarchies.readOnly().getAccreditationsToAccredit(federation.id, receiver);
    assert(accreditationsToAccredit.accreditations.length > 0, "Receiver should have permission to accredit");
    console.log("\n✅ Accreditation to accredit found for receiver");

    // Revoke the permission
    const permissionId = accreditationsToAccredit.accreditations[0].id;

    await hierarchies
        .revokeAccreditationToAccredit(federation.id, receiver, permissionId)
        .buildAndExecute(hierarchies);
    console.log("\n✅ Accreditation to accredit revoked successfully");

    // Check if the permission was revoked
    accreditationsToAccredit = await hierarchies.readOnly().getAccreditationsToAccredit(federation.id, receiver);
    assert(accreditationsToAccredit.accreditations.length === 0, "Accreditation not revoked for receiver");
    console.log("\n✅ Accreditation successfully revoked for receiver");
}
