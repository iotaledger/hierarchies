// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, HierarchiesClient, FederationProperty, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import assert from "assert";
import { randomBytes } from "crypto";
import { getFundedClient } from "./util";

/**
 * Demonstrate how to revoke an accreditation to attest to a Property.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function revokeAccreditationToAttest(client?: HierarchiesClient) {
    console.log("\nRunning revoke accreditation to attest example");

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

    const allowedValues = [value];

    // Add the Property to the federation
    await hierarchies
        .addProperty(federation.id, propertyName, allowedValues, false)
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Property ${propertyName.dotted()} added successfully`);

    // A receiver is an account that will receive the attestation
    const receiver = "0x" + randomBytes(32).toString("hex");

    // Property
    const property = new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("Hello")]);

    // Let us issue an accreditation to attest to the Property
    await hierarchies
        .createAccreditationToAttest(federation.id, receiver, [property])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest issued successfully for ${receiver}`);

    // Check if the accreditation was issued
    let accreditationsToAttest = await hierarchies.readOnly().getAccreditationsToAttest(federation.id, receiver);
    assert(accreditationsToAttest.accreditations.length > 0, "Accreditation not found for receiver");
    console.log("\n✅ Accreditation to attest found for receiver");

    // Revoke the accreditation
    const permissionId = accreditationsToAttest.accreditations[0].id;

    await hierarchies.revokeAccreditationToAttest(federation.id, receiver, permissionId).buildAndExecute(hierarchies);
    console.log("\n✅ Accreditation to attest revoked successfully");

    // Check if the accreditation was revoked
    accreditationsToAttest = await hierarchies.readOnly().getAccreditationsToAttest(federation.id, receiver);
    assert(accreditationsToAttest.accreditations.length === 0, "Accreditation was not revoked");
    console.log("\n✅ Accreditation successfully revoked for receiver");
}
