// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { createFederation } from "./01_create_federation";
import { addRootAuthority } from "./02_add_root_authority";
import { addStatement } from "./03_add_statement";
import { createAccreditationToAttest } from "./04_create_accreditation_to_attest";
import { revokeAccreditationToAttest } from "./05_revoke_accreditation_to_attest";
import { createAccreditationToAccredit } from "./06_create_accreditation_to_accredit";
import { revokeAccreditationToAccredit } from "./07_revoke_accreditation_to_accredit";

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
        case "03_add_statement":
            return await addStatement();
        case "04_create_accreditation_to_attest":
            return await createAccreditationToAttest();
        case "05_revoke_accreditation_to_attest":
            return await revokeAccreditationToAttest();
        case "06_create_accreditation_to_accredit":
            return await createAccreditationToAccredit();
        case "07_revoke_accreditation_to_accredit":
            return await revokeAccreditationToAccredit();
        default:
            throw "Unknown example name: '" + argument + "'";
    }
}

main()
    .catch((error) => {
        console.log("Example error:", error);
    });
