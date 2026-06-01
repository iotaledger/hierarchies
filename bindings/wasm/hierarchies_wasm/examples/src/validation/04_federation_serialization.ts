// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    Accreditations,
    Federation,
    FederationProperty,
    PropertyName,
    PropertyShape,
    PropertyValue,
} from "@iota/hierarchies/node";
import { strict as assert } from "assert";
import { generateRandomAddress, getFundedClient } from "../util";

/**
 * Regression test for the `getFederationById` serialization inconsistency.
 *
 * Previously the `governance.accreditationsToAttest` /
 * `accreditationsToAccredit` Map values were produced via
 * `serde_wasm_bindgen::to_value`, which bypassed the `wasm_bindgen` getters and
 * leaked raw Rust field names (`accredited_by`, `allow_any`, `valid_from_ms`,
 * …). This asserts the Map values now expose the same camelCase getters as the
 * rest of the response, and that the inner property wrappers surface their data
 * (instead of rendering as `{}`).
 */
export async function federationSerialization(): Promise<void> {
    const hierarchies = await getFundedClient();
    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(
        hierarchies,
    );
    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    const propertyName = new PropertyName(["test", "property"]);

    await hierarchies.addProperty(
        federation.id,
        new FederationProperty(propertyName).withAllowedValues([PropertyValue.newText("foo")]),
    ).buildAndExecute(hierarchies);
    console.log(`\n✅ Property ${propertyName.dotted()} added successfully`);

    const receiver = generateRandomAddress();
    const propertyToAttest = new FederationProperty(propertyName)
        .withAllowedValues([PropertyValue.newText("foo")])
        .withCondition(PropertyShape.newContains("foo"));

    await hierarchies.createAccreditationToAttest(federation.id, receiver, [propertyToAttest])
        .buildAndExecute(hierarchies);
    console.log(`\n✅ Accreditation to attest created for ${receiver}`);

    // Fetch the whole federation — this is the path that used to mix casing.
    const fetched: Federation = await hierarchies.readOnly().getFederationById(federation.id);

    const attestMap = fetched.governance.accreditationsToAttest;
    assert(attestMap.size > 0, "Expected at least one accreditationsToAttest entry");

    // The Map value must be a real `Accreditations` wrapper, not a serde dump.
    const accreditations = [...attestMap.values()][0] as Accreditations;
    assert(accreditations.accreditations.length > 0, "Expected at least one accreditation");

    const accreditation = accreditations.accreditations[0];

    // (A) camelCase getters must work — raw snake_case must NOT be present.
    assert.equal(typeof accreditation.accreditedBy, "string", "accreditedBy getter should return a string");
    assert.equal(
        (accreditation as unknown as Record<string, unknown>).accredited_by,
        undefined,
        "snake_case `accredited_by` must not leak through the Map path",
    );

    // properties is an Array<FederationProperty>, matching governance.properties.data.
    assert(Array.isArray(accreditation.properties), "properties should be an array");
    assert(accreditation.properties.length > 0, "Expected at least one property");
    const property = accreditation.properties[0];

    assert.equal(typeof property.allowAny, "boolean", "allowAny getter should return a boolean");
    assert.equal(
        (property as unknown as Record<string, unknown>).allow_any,
        undefined,
        "snake_case `allow_any` must not leak",
    );

    // timespan getters are camelCase (validFromMs / validUntilMs).
    assert(property.timespan !== undefined, "timespan should be present");
    assert.equal(
        (property.timespan as unknown as Record<string, unknown>).valid_from_ms,
        undefined,
        "snake_case `valid_from_ms` must not leak",
    );

    // (B) inner wrappers must surface their data, not render as `{}`.
    assert(property.propertyName.getNames().length > 0, "propertyName should expose its names");
    assert.equal(property.propertyName.dotted(), propertyName.dotted(), "propertyName should match");

    const condition = property.condition;
    assert(condition !== undefined, "condition (shape) should be present");
    assert.equal(condition!.asContains(), "foo", "condition should expose its Contains value");

    // The new `type`/`value` getters and `names` getter make the data
    // enumerable, so JSON.stringify no longer hides it behind `{}`.
    const json = JSON.parse(JSON.stringify(accreditation));
    assert(!("accredited_by" in json), "serialized accreditation must not contain snake_case keys");

    console.log("✅ getFederationById accreditations expose consistent camelCase getters and visible inner data");
}
