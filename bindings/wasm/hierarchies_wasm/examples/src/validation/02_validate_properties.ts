// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, FederationProperty, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import assert from "assert";
import { randomBytes } from "crypto";
import { getFundedClient } from "../util";

export async function validateProperties(): Promise<void> {
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

    const accreditationReceiver = "0x" + randomBytes(32).toString("hex");
    const propertyToAttest = new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("Hello")]);

    // Create an accreditation to attest
    await hierarchies.createAccreditationToAttest(federation.id, accreditationReceiver, [propertyToAttest])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest created for ${accreditationReceiver}`);

    // Validate the properties
    const properties = new Map<PropertyName, PropertyValue>([[propertyName, propertyValue]]);
    const validationResult = await hierarchies.readOnly().validateProperties(
        federation.id,
        accreditationReceiver,
        properties,
    );

    assert(validationResult, "Validation failed");

    console.log("\n✅ Successfully validated properties for the receiver:", accreditationReceiver);
}
