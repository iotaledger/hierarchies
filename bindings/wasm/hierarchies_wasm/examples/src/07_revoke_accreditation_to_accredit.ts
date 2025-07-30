// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    Statement,
    StatementName,
    StatementValue,
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
    const hierarchies = client ?? (await getFundedClient());

    // Create a new federation
    const { output: federation } = await hierarchies.createNewFederation().buildAndExecute(hierarchies);

    // The ID of the federation
    const federationId = federation.id;

    // The name of the statement
    const statementName = new StatementName(["Example LTD"]);

    // The value of the statement
    const value = StatementValue.fromText("Hello");

    const allowedValues = [value];

    console.log("Adding statement");

    // Add the statement to the federation
    await hierarchies
        .addStatement(federationId, statementName, allowedValues, false)
        .buildAndExecute(hierarchies);

    console.log("Added statement");

    // A receiver is an account that will receive the accreditation
    const receiver = "0x" + randomBytes(32).toString("hex");
    const allowedValuesAccreditation = [ StatementValue.fromText("Hello")];

    // Statements
    const statement = new Statement(statementName).withAllowedValues([StatementValue.fromText("Hello")]);

    // Let us issue a permission to accredit to the Statement
    await hierarchies
        .createAccreditationToAccredit(federationId, receiver, [statement])
        .buildAndExecute(hierarchies);

    console.log("Issued permission to accredit");

    console.log("Checking if the receiver has the permission to accredit");
    // Check if the receiver has the permission to accredit
    let federationData = await hierarchies.readOnly().getFederationById(federationId);
    const canAccredit = federationData.governance.accreditationsToAccredit.has(receiver);
    if (!canAccredit) {
        throw new Error("Receiver should have permission to accredit");
    }

    // Revoke the permission
    const accreditationsToAccredit = await hierarchies.readOnly().getAccreditationsToAccredit(federationId, receiver);
    if (accreditationsToAccredit.accreditations.length === 0) {
        throw new Error("No statements found to revoke");
    }
    const permissionId = accreditationsToAccredit.accreditations[0].id;

    await hierarchies
        .revokeAccreditationToAccredit(federationId, receiver, permissionId)
        .buildAndExecute(hierarchies);

    console.log("Revoked permission to accredit");

    // Check if the permission was revoked
    federationData = await hierarchies.readOnly().getFederationById(federationId);

    console.log("Federation:", federationData);

    const accreditations = federationData.governance.accreditationsToAccredit.get(receiver);

    if (accreditations && accreditations.statements.length > 0) {
        throw new Error("Accreditation not revoked for receiver");
    }

    console.log("Accreditation successfully revoked for receiver");
}
