// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation } from "@iota/hierarchies/node";
import assert from "assert";
import { getFundedClient } from "./util";

export async function createFederation(): Promise<void> {
    const hierarchies = await getFundedClient();

    const { output: federation }: { output: Federation } = await hierarchies.createNewFederation().buildAndExecute(
        hierarchies,
    );

    console.log("\n✅ Federation created successfully!");
    console.log("Federation ID: ", federation.id);

    assert(federation.id, "Federation ID should not be empty");
    assert(federation.rootAuthorities.length > 0, "Federation should have at least one root authority");
}
