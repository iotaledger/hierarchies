// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, Statement, StatementName, StatementValue } from "@iota/hierarchies/node";
import { getFundedClient } from "../util";
import { randomBytes } from "crypto";
import assert from "assert";

export async function validateStatements(): Promise<void> {
    const hierarchies = await getFundedClient();
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(hierarchies);

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    const statementName = new StatementName(["Example LTD"]);
    const statementValue = StatementValue.newText("Hello");
    const allowedValues = [statementValue];

    await hierarchies.addStatement(federation.id, statementName, allowedValues, false)
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Statement ${statementName.dotted()} added successfully`);


    const accreditationReceiver = "0x" + randomBytes(32).toString("hex");
    const statementToAttest = new Statement(statementName).withAllowedValues([StatementValue.newText("Hello")]);

    // Create an accreditation to attest
    await hierarchies.createAccreditationToAttest(federation.id, accreditationReceiver, [statementToAttest])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest created for ${accreditationReceiver}`);


    // Validate the statements
    const statements = new Map<StatementName, StatementValue>([[statementName, statementValue]]);
    const validationResult = await hierarchies.readOnly().validateStatements(federation.id, accreditationReceiver, statements);

    assert(validationResult, "Validation failed");

    console.log("\n✅ Successfully validated statements for the receiver:", accreditationReceiver);
}
