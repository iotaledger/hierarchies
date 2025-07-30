import { Federation, Statement, StatementName, StatementValue } from "@iota/hierarchies/node";
import { getFundedClient } from "../util";
import { randomBytes } from "crypto";
import assert from "assert";

export async function getAccreditations(): Promise<void> {
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

    const receiver = "0x" + randomBytes(32).toString("hex");

    // Create and get the accreditation to attest

    const statementToAttest = new Statement(statementName).withAllowedValues([StatementValue.newText("Hello")]);

    await hierarchies.createAccreditationToAttest(federation.id, receiver, [statementToAttest])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest created for ${receiver}`);


    const accreditationsToAttest = await hierarchies.readOnly().getAccreditationsToAttest(federation.id, receiver);
    assert(accreditationsToAttest.accreditations.length > 0, "No accreditations to attest found");
    assert(accreditationsToAttest.accreditations[0].statements.length > 0, "No statements found in accreditation to attest");
    assert(accreditationsToAttest.accreditations[0].statements[0].statementName.dotted() === statementName.dotted(), "Statement name does not match for accreditation to attest");

    console.log("✅ Successfully retrieved accreditations to attest for the receiver:", receiver);


    // Create and get the accreditation to accredit

    const statementToAccredit = new Statement(statementName).withAllowedValues([StatementValue.newText("Hello")]);

    await hierarchies.createAccreditationToAccredit(federation.id, receiver, [statementToAccredit])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to accredit created for ${receiver}`);

    const accreditationsToAccredit = await hierarchies.readOnly().getAccreditationsToAccredit(federation.id, receiver);
    assert(accreditationsToAccredit.accreditations.length > 0, "No accreditations to accredit found");
    assert(accreditationsToAccredit.accreditations[0].statements.length > 0, "No statements found in accreditation to accredit");
    assert(accreditationsToAccredit.accreditations[0].statements[0].statementName.dotted() === statementName.dotted(), "Statement name does not match for accreditation to accredit");

    console.log("✅ Successfully retrieved accreditations to accredit for the receiver:", receiver);
}
