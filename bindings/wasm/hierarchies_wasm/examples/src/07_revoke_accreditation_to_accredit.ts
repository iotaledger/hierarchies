// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, FederationProperty, HierarchiesClient, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import assert from "assert";
import { randomBytes } from "crypto";
import { getFundedClient } from "./util";

/**
 * Demonstrate how to revoke an accreditation to accredit to a Property.
 *
 * In this example we connect to a locally running private network, but it can
 * be adapted to run on any IOTA node by setting the network and faucet
 * endpoints.
 */
export async function revokeAccreditationToAccredit(client?: HierarchiesClient) {
    console.log("\nRunning revoke accreditation to accredit example");

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

    // A receiver is an account that will receive the accreditation
    const receiver = "0x" + randomBytes(32).toString("hex");

    // Property
    const property = new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("Hello")]);

    // Let us issue an accreditation to accredit to the Property
    await hierarchies
        .createAccreditationToAccredit(federation.id, receiver, [property])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to accredit issued successfully for ${receiver}`);

    // Check if the receiver has the permission to accredit
    let accreditationsToAccredit = await hierarchies.readOnly().getAccreditationsToAccredit(federation.id, receiver);
    assert(accreditationsToAccredit.accreditations.length > 0, "Receiver should have permission to accredit");
    console.log("\n✅ Accreditation to accredit found for receiver");

    // Revoke the accreditation
    const permissionId = accreditationsToAccredit.accreditations[0].id;

    await hierarchies
        .revokeAccreditationToAccredit(federation.id, receiver, permissionId)
        .buildAndExecute(hierarchies);
    console.log("\n✅ Accreditation to accredit revoked successfully");

    // Check if the accreditation was revoked
    accreditationsToAccredit = await hierarchies.readOnly().getAccreditationsToAccredit(federation.id, receiver);
    assert(accreditationsToAccredit.accreditations.length === 0, "Accreditation not revoked for receiver");
    console.log("\n✅ Accreditation successfully revoked for receiver");
}
