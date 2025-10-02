import { Federation, FederationProperty, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import { strict as assert } from "assert";
import { generateRandomAddress, getFundedClient } from "../util";
export async function getAccreditations(): Promise<void> {
    const hierarchies = await getFundedClient();
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(
        hierarchies,
    );

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    const propertyName = new PropertyName(["Example LTD"]);
    const propertyValue = PropertyValue.newText("Hello");

    await hierarchies.addProperty(
        federation.id,
        new FederationProperty(propertyName).withAllowedValues([propertyValue]),
    )
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Property ${propertyName.dotted()} added successfully`);

    const receiver = generateRandomAddress();

    // Create and get the accreditation to attest
    const propertyToAttest = new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("Hello")]);

    await hierarchies.createAccreditationToAttest(federation.id, receiver, [propertyToAttest])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest created for ${receiver}`);

    const accreditationsToAttest = await hierarchies.readOnly().getAccreditationsToAttest(federation.id, receiver);
    assert(accreditationsToAttest.accreditations.length > 0, "No accreditations to attest found");
    assert(
        accreditationsToAttest.accreditations[0].properties.length > 0,
        "No properties found in accreditation to attest",
    );
    assert.equal(
        accreditationsToAttest.accreditations[0].properties[0].propertyName.dotted(),
        propertyName.dotted(),
        "Property name does not match for accreditation to attest",
    );

    console.log("✅ Successfully retrieved accreditations to attest for the receiver:", receiver);

    // Create and get the accreditation to accredit

    const propertyToAccredit = new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("Hello")]);

    await hierarchies.createAccreditationToAccredit(federation.id, receiver, [propertyToAccredit])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to accredit created for ${receiver}`);

    const accreditationsToAccredit = await hierarchies.readOnly().getAccreditationsToAccredit(federation.id, receiver);
    assert(accreditationsToAccredit.accreditations.length > 0, "No accreditations to accredit found");
    assert(
        accreditationsToAccredit.accreditations[0].properties.length > 0,
        "No properties found in accreditation to accredit",
    );
    assert.equal(
        accreditationsToAccredit.accreditations[0].properties[0].propertyName.dotted(),
        propertyName.dotted(),
        "Property name does not match for accreditation to accredit",
    );

    console.log("✅ Successfully retrieved accreditations to accredit for the receiver:", receiver);
}
