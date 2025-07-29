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
 * Demonstrate how to issue a permission to accredit to a Statement.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function createAccreditationToAccredit(client?: HierarchiesClient) {
    console.log("Running create accreditation to accredit example");

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

    // Check if the permission was issued
    const federationData = await ithClient.readOnly().getFederationById(federationId);

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
