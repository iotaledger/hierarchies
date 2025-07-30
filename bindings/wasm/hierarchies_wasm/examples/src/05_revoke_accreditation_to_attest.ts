// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    HierarchiesClient,
    Statement,
    StatementName,
    StatementValue,
} from "@iota/hierarchies/node";
import { randomBytes } from "crypto";
import { getFundedClient } from "./util";

/**
 * Demonstrate how to revoke a permission to attest to a Statement.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function revokeAccreditationToAttest(client?: HierarchiesClient) {
    console.log("Running revoke accreditation to attest example");

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

    // A receiver is an account that will receive the attestation
    const receiver = "0x" + randomBytes(32).toString("hex");
    const allowedValuesAccreditation = [StatementValue.fromText("Hello")];

    // Statements
    const statement = new Statement(statementName).withAllowedValues([StatementValue.fromText("Hello")]);

    // Let us issue a permission to attest to the Statement
    await hierarchies
        .createAccreditationToAttest(federationId, receiver, [statement])
        .buildAndExecute(hierarchies);

    console.log("Issued permission to attest");

    // Check if the permission was issued
    let federationData = await hierarchies.readOnly().getFederationById(federationId);

    // Check if the receiver has the permission to attest
    const canAccredit = federationData.governance.accreditationsToAttest.has(
        receiver,
    );
    if (!canAccredit) {
        throw new Error("Accreditation not found for receiver");
    }
    console.log("Accreditation found for receiver");

    // Revoke the permission
    const accreditationsToAttest = await hierarchies.readOnly().getAccreditationsToAttest(federationId, receiver);

    if (accreditationsToAttest.accreditations.length === 0) {
        throw new Error("No statements found to revoke");
    }

    const permissionId = accreditationsToAttest.accreditations[0].id;

    await hierarchies.revokeAccreditationToAttest(federationId, receiver, permissionId).buildAndExecute(hierarchies);

    console.log("Revoked permission to attest");

    // Check if the permission was revoked
    federationData = await hierarchies.readOnly().getFederationById(federationId);

    const receiverAccreditations = federationData.governance.accreditationsToAttest.get(receiver);

    if (receiverAccreditations && receiverAccreditations.statements.length > 0) {
        throw new Error("Accreditation not revoked");
    }

    console.log("Accreditation successfully revoked for receiver");
}
