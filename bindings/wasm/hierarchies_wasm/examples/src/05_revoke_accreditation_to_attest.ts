// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    Federation,
    HierarchiesClient,
    Statement,
    StatementName,
    StatementValue,
} from "@iota/hierarchies/node";
import { randomBytes } from "crypto";
import { getFundedClient } from "./util";
import assert from "assert";

/**
 * Demonstrate how to revoke a permission to attest to a Statement.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function revokeAccreditationToAttest(client?: HierarchiesClient) {
    console.log("\nRunning revoke accreditation to attest example");

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


    // A receiver is an account that will receive the attestation
    const receiver = "0x" + randomBytes(32).toString("hex");

    // Statements
    const statement = new Statement(statementName).withAllowedValues([StatementValue.newText("Hello")]);

    // Let us issue a permission to attest to the Statement
    await hierarchies
        .createAccreditationToAttest(federation.id, receiver, [statement])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest issued successfully for ${receiver}`);


    // Check if the permission was issued
    let accreditationsToAttest = await hierarchies.readOnly().getAccreditationsToAttest(federation.id, receiver);
    assert(accreditationsToAttest.accreditations.length > 0, "Accreditation not found for receiver");
    console.log("\n✅ Accreditation to attest found for receiver");

    // Revoke the permission
    const permissionId = accreditationsToAttest.accreditations[0].id;

    await hierarchies.revokeAccreditationToAttest(federation.id, receiver, permissionId).buildAndExecute(hierarchies);
    console.log("\n✅ Accreditation to attest revoked successfully");

    // Check if the permission was revoked
    accreditationsToAttest = await hierarchies.readOnly().getAccreditationsToAttest(federation.id, receiver);
    assert(accreditationsToAttest.accreditations.length === 0, "Accreditation was not revoked");
    console.log("\n✅ Accreditation successfully revoked for receiver");
}
