// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, FederationProperty, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import { HierarchiesClient } from "@iota/hierarchies/node";
import { strict as assert } from "assert";
import { generateRandomAddress, getFundedClient } from "./util";

/**
 * Demonstrate how to issue an accreditation to attest to a Property.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function createAccreditationToAttest(client?: HierarchiesClient) {
    console.log("\nRunning create accreditation to attest example");

    // Get the client instance
    const hierarchies = client ?? (await getFundedClient());

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(
        hierarchies,
    );
    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    // Federation property name
    const propertyName = new PropertyName(["Example LTD"]);

    // Federation property value
    const value = PropertyValue.newText("Hello");

    // Add the Property to the federation
    await hierarchies
        .addProperty(federation.id, new FederationProperty(propertyName).withAllowedValues([value]))
        .buildAndExecute(hierarchies);

    console.log(`\n✅ Property ${propertyName.dotted()} added successfully`);

    // A receiver is an account that will receive the attestation
    const receiver = generateRandomAddress();

    // Property
    const property = new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("Hello")]);

    // Let us issue an accreditation to attest to the Property
    await hierarchies
        .createAccreditationToAttest(federation.id, receiver, [property])
        .buildAndExecute(hierarchies);

    console.log(`\n✅ Accreditation to attest issued successfully for ${receiver}`);

    // Check if the accreditation was issued
    const accreditationsToAttest = await hierarchies.readOnly().getAccreditationsToAttest(federation.id, receiver);

    assert(accreditationsToAttest.accreditations.length > 0, "Accreditation not found for receiver");
    assert.equal(
        accreditationsToAttest.accreditations[0].properties[0].propertyName.dotted(),
        propertyName.dotted(),
        "Property name does not match",
    );

    console.log("\n✅ Accreditation to attest found for receiver");
}
