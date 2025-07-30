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
 * Demonstrate how to issue a permission to accredit to a Statement.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function createAccreditationToAccredit(client?: HierarchiesClient) {
    console.log("Running create accreditation to accredit example");

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

    // Check if the permission was issued
    const federationData = await hierarchies.readOnly().getFederationById(federationId);

    console.log("Federation:", federationData);

    // Check if the receiver has the permission to accredit
    const hasPermission = federationData.governance.accreditationsToAccredit.has(
        receiver,
    );

    if (!hasPermission) {
        throw new Error("Accreditation not found for receiver");
    }

    console.log("Accreditation found for receiver");
}
