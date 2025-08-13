// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, Statement, StatementName, StatementValue } from "@iota/hierarchies/node";
import assert from "assert";
import { randomBytes } from "crypto";
import { getFundedClient } from "../util";

export async function getStatements(): Promise<void> {
    const hierarchies = await getFundedClient();
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(
        hierarchies,
    );

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    const statementName = new StatementName(["Example LTD"]);
    const statementValue = StatementValue.newText("Hello");
    const allowedValues = [statementValue];

    await hierarchies.addStatement(federation.id, statementName, allowedValues, false)
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Statement ${statementName.dotted()} added successfully`);

    const statementToAttest = new Statement(statementName).withAllowedValues([StatementValue.newText("Hello")]);
    const accreditationReceiver = "0x" + randomBytes(32).toString("hex");

    // Create an accreditation to attest
    await hierarchies.createAccreditationToAttest(federation.id, accreditationReceiver, [statementToAttest])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest created for ${accreditationReceiver}`);

    // Get the statements
    const retrievedStatements = await hierarchies.readOnly().getStatements(federation.id);

    assert(retrievedStatements.length > 0, "No statements found");
    console.log("\n✅ Successfully retrieved statements for the receiver:", accreditationReceiver);
}
