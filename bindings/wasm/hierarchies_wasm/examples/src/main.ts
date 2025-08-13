// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { createFederation } from "./01_create_federation";
import { addRootAuthority } from "./02_add_root_authority";
import { addStatement } from "./03_add_statement";
import { createAccreditationToAttest } from "./04_create_accreditation_to_attest";
import { revokeAccreditationToAttest } from "./05_revoke_accreditation_to_attest";
import { createAccreditationToAccredit } from "./06_create_accreditation_to_accredit";
import { revokeAccreditationToAccredit } from "./07_revoke_accreditation_to_accredit";
import { revokeRootAuthority } from "./08_revoke_root_authority";
import { getAccreditations } from "./validation/01_get_accreditations";
import { validateStatements } from "./validation/02_validate_statements";
import { getStatements } from "./validation/03_get_statements";

export async function main(example?: string) {
    // Extract example name.
    const argument = example ?? process.argv?.[2]?.toLowerCase();
    if (!argument) {
        throw "Please specify an example name, e.g. '01_create_federation'";
    }

    switch (argument) {
        case "all":
            await createFederation();
            await addRootAuthority();
            await addStatement();
            await createAccreditationToAttest();
            await revokeAccreditationToAttest();
            await createAccreditationToAccredit();
            await revokeAccreditationToAccredit();
            await revokeRootAuthority();
            await getAccreditations();
            await validateStatements();
            return await getStatements();
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
        case "08_revoke_root_authority":
            return await revokeRootAuthority();
        case "01_get_accreditations":
            return await getAccreditations();
        case "02_validate_statements":
            return await validateStatements();
        case "03_get_statements":
            return await getStatements();
        default:
            throw "Unknown example name: '" + argument + "'";
    }
}

main()
    .catch((error) => {
        console.log("Example error:", error);
    });
