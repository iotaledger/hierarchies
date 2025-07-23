// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Federation } from "@iota/hierarchies/node";
import { getFundedClient } from "./util";

export async function createFederation(): Promise<void> {
    const hierarchiesClient = await getFundedClient();

    const {output: federation}: {output: Federation}  = await hierarchiesClient.createNewFederation().buildAndExecute(hierarchiesClient);

    console.log("\n✅ Federation created successfully!");
    console.log("Notarization ID: ", federation.id);
    console.log("Notarization Method: ", federation.rootAuthorities);
}
