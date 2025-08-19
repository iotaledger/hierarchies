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

    await hierarchies.addProperty(
        federation.id,
        new FederationProperty(propertyName).withAllowedValues([propertyValue]),
    )
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Property ${propertyName.dotted()} added successfully`);

    const secondProperty = new FederationProperty(new PropertyName(["Example LTD 2", "Example LTD 3"]))
        .withAllowedValues([PropertyValue.newText("Hello 2")]);

    // Add a second property
    await hierarchies.addProperty(federation.id, secondProperty).buildAndExecute(hierarchies);
    console.log(`\n✅ Property ${secondProperty.propertyName.dotted()} added successfully`);

    // Get the properties
    const retrievedProperties = await hierarchies.readOnly().getProperties(federation.id);

    assert(retrievedProperties.length > 0, "No properties found");
    console.log("\n✅ Successfully retrieved properties for the Federation", retrievedProperties);
}
