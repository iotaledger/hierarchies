// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import assert from "assert";
import { getFundedClient } from "./util";

/**
 * Demonstrates how to add a Property to a federation.
 */
export async function addProperty(): Promise<void> {
    // Get the client instance
    const hierarchies = await getFundedClient();

    // Create a new federation
    const { output: federation }: { output: Federation } = await hierarchies
        .createNewFederation()
        .buildAndExecute(hierarchies);

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    // Federation property name
    const propertyName = new PropertyName(["company", "example"]);

    // Federation property values
    const value = PropertyValue.newText("Hello");
    const anotherValue = PropertyValue.newText("World");
    const allowedValues = [value, anotherValue];

    // Add the Property to the federation
    await hierarchies
        .addProperty(federation.id, propertyName, allowedValues, false)
        .buildAndExecute(hierarchies);

    console.log(`\n✅ Property: ${propertyName.dotted()} was added successfully.`);

    // Get the updated federation
    const updatedFederation: Federation = await hierarchies.readOnly().getFederationById(federation.id);

    const addedProperty = updatedFederation.governance.properties.data.find(p =>
        p.propertyName.dotted() === propertyName.dotted()
    );
    assert(addedProperty, `Didn't find the Property in the Federation: ${propertyName.dotted()}`);

    console.log("\n✅ Property was successfully added to the federation.");
}
