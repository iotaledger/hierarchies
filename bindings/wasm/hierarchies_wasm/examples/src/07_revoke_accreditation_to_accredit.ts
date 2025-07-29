// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    Statement,
    StatementName,
    StatementValue,
    Timespan,
} from "@iota/hierarchies/node";
import { getFundedClient } from "./util";
import { HierarchiesClient } from "@iota/hierarchies/node";
import { randomBytes } from "crypto";

/**
 * Demonstrate how to revoke a permission to accredit to a Statement.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function revokeAccreditationToAccredit(client?: HierarchiesClient) {
    console.log("Running revoke accreditation to accredit example");

    // Get the client instance
    const ithClient = client ?? (await getFundedClient());

    // Create a new federation
    const { output: federation } = await ithClient.createNewFederation().buildAndExecute(ithClient);

    // The ID of the federation
    const federationId = federation.id;

    // The name of the statement
    const statementName = new StatementName(["Example LTD"]);

    // The value of the statement
    const value = StatementValue.newText("Hello");

    const allowedValues = [value];

    console.log("Adding statement");

    // Add the statement to the federation
    await ithClient
        .addStatement(federationId, statementName, allowedValues, false)
        .buildAndExecute(ithClient);

    console.log("Added statement");

    // A receiver is an account that will receive the accreditation
    const receiver = "0x" + randomBytes(32).toString("hex");
    const allowedValuesAccreditation = [ StatementValue.newText("Hello")];

    // Statements
    const statement = new Statement(
            statementName,
            allowedValuesAccreditation,
            undefined,
            false,
            new Timespan()
          );

    // Let us issue a permission to accredit to the Statement
    await ithClient
        .createAccreditationToAccredit(federationId, receiver, [statement])
        .buildAndExecute(ithClient);

    console.log("Issued permission to accredit");

    console.log("Checking if the receiver has the permission to accredit");
    // Check if the receiver has the permission to accredit
    let federationData = await ithClient.readOnly().getFederationById(federationId);
    const canAccredit = federationData.governance.accreditationsToAccredit.has(receiver);
    if (!canAccredit) {
        throw new Error("Receiver should have permission to accredit");
    }

    // Revoke the permission
    const statements = await ithClient.readOnly().getAccreditationsToAccredit(federationId, receiver);
    if (statements.statements.length === 0) {
        throw new Error("No statements found to revoke");
    }
    const permissionId = statements.statements[0].id;

    await ithClient
        .revokeAccreditationToAccredit(federationId, receiver, permissionId)
        .buildAndExecute(ithClient);

    console.log("Revoked permission to accredit");

    // Check if the permission was revoked
    federationData = await ithClient.readOnly().getFederationById(federationId);

    console.log("Federation:", federationData);

    const accreditations = federationData.governance.accreditationsToAccredit.get(receiver);

    if (accreditations && accreditations.statements.length > 0) {
        throw new Error("Accreditation not revoked for receiver");
    }

    console.log("Accreditation successfully revoked for receiver");
}
