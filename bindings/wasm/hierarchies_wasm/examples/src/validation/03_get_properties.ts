// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, FederationProperty, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import assert from "assert";
import { randomBytes } from "crypto";
import { getFundedClient } from "../util";

export async function getProperties(): Promise<void> {
    const hierarchies = await getFundedClient();
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(
        hierarchies,
    );

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    const propertyName = new PropertyName(["Example LTD"]);
    const propertyValue = PropertyValue.newText("Hello");
    const allowedValues = [propertyValue];

    await hierarchies.addProperty(federation.id, propertyName, allowedValues, false)
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Property ${propertyName.dotted()} added successfully`);

    const propertyToAttest = new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("Hello")]);
    const accreditationReceiver = "0x" + randomBytes(32).toString("hex");

    // Create an accreditation to attest
    await hierarchies.createAccreditationToAttest(federation.id, accreditationReceiver, [propertyToAttest])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest created for ${accreditationReceiver}`);

    // Get the properties
    const retrievedProperties = await hierarchies.readOnly().getProperties(federation.id);

    assert(retrievedProperties.length > 0, "No properties found");
    console.log("\n✅ Successfully retrieved properties for the receiver:", accreditationReceiver);
}
