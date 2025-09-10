// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { createFederation } from "./01_create_federation";
import { addRootAuthority } from "./02_add_root_authority";
import { addProperties } from "./03_add_properties";
import { createAccreditationToAttest } from "./04_create_accreditation_to_attest";
import { revokeAccreditationToAttest } from "./05_revoke_accreditation_to_attest";
import { createAccreditationToAccredit } from "./06_create_accreditation_to_accredit";
import { revokeAccreditationToAccredit } from "./07_revoke_accreditation_to_accredit";
import { revokeRootAuthority } from "./08_revoke_root_authority";
import { reinstateRootAuthority } from "./09_reinstate_root_authority";
import { getAccreditations } from "./validation/01_get_accreditations";
import { validateProperties } from "./validation/02_validate_properties";
import { getProperties } from "./validation/03_get_properties";

import { afterEach } from "mocha";

// Only verifies that no uncaught exceptions are thrown, including syntax errors etc.
describe("Test node examples", function() {
    afterEach(
        () => {
            console.log("\n----------------------------------------------------\n");
        },
    );
    it("Should create Federation", async () => {
        await createFederation();
    });
    it("Should add Root Authority", async () => {
        await addRootAuthority();
    });
    it("Should add Properties", async () => {
        await addProperties();
    });
    it("Should create Accreditation to Attest", async () => {
        await createAccreditationToAttest();
    });
    it("Should revoke Accreditation to Attest", async () => {
        await revokeAccreditationToAttest();
    });
    it("Should create Accreditation to Accredit", async () => {
        await createAccreditationToAccredit();
    });
    it("Should revoke Accreditation to Accredit", async () => {
        await revokeAccreditationToAccredit();
    });
    it("Should revoke Root Authority", async () => {
        await revokeRootAuthority();
    });
    it("Should reinstate Root Authority", async () => {
        await reinstateRootAuthority();
    });
    it("Should get Accreditations", async () => {
        await getAccreditations();
    });
    it("Should validate Properties", async () => {
        await validateProperties();
    });
    it("Should get Properties", async () => {
        await getProperties();
    });
});
