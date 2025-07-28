// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { createFederation  } from "./01_create_federation";
import { addRootAuthority } from "./02_add_root_authority";

export async function main(example?: string) {
    // Extract example name.
    const argument = example ?? process.argv?.[2]?.toLowerCase();
    if (!argument) {
        throw "Please specify an example name, e.g. '01_create_federation'";
    }

    switch (argument) {
        case "01_create_federation":
            return await createFederation();
        case "02_add_root_authority":
            return await addRootAuthority();
        default:
            throw "Unknown example name: '" + argument + "'";
    }
}

main()
    .catch((error) => {
        console.log("Example error:", error);
    });
