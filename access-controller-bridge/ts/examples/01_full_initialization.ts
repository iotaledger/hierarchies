// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Full initialization of the Access Controller Bridge.
//
// Run:
//   IOTA_HIERARCHIES_PKG_ID=0x... IOTA_AUDIT_TRAIL_PKG_ID=0x... \
//   IOTA_TF_COMPONENTS_PKG_ID=0x... IOTA_ACB_PKG_ID=0x... \
//   npx ts-node 01_full_initialization.ts

import { Transaction } from "@iota/iota-sdk/transactions";
import { PtbHelper, getClientAndSigner, executeTx, findCreated } from "./utils";

async function main() {
    const h = new PtbHelper();
    const { client, keypair, address } = await getClientAndSigner();

    // ===== Phase 1: Create federation + property =====
    console.log("Phase 1: Creating federation...");

    let tx = new Transaction();
    h.newFederation(tx);
    let changes = await executeTx(client, keypair, tx);
    const federationId = findCreated(changes, "Federation")[0];
    const rootCapId = findCreated(changes, "RootAuthorityCap")[0];
    const accreditCapId = findCreated(changes, "AccreditCap")[0];
    console.log(`  Federation: ${federationId}`);

    tx = new Transaction();
    h.fedAddProperty(tx, federationId, rootCapId, "catch_logging", ["Cod", "Haddock"], false);
    await executeTx(client, keypair, tx);
    console.log("  catch_logging property added");

    // ===== Phase 2: Create audit trail =====
    console.log("\nPhase 2: Creating audit trail...");

    tx = new Transaction();
    h.trailCreate(tx, "ACB Test Trail", address);
    changes = await executeTx(client, keypair, tx);
    const trailId = findCreated(changes, "AuditTrail")[0];
    const adminCapId = findCreated(changes, "Capability")[0];
    console.log(`  AuditTrail: ${trailId}`);

    // ===== Phase 3: Create role =====
    console.log("\nPhase 3: Creating role...");

    tx = new Transaction();
    h.trailCreateRole(tx, trailId, adminCapId, "catch_logger", ["add_record"]);
    await executeTx(client, keypair, tx);
    console.log("  catch_logger role created");

    // ===== Phase 4: Mint capability =====
    console.log("\nPhase 4: Minting capability...");

    tx = new Transaction();
    h.trailMintCapability(tx, trailId, adminCapId, "catch_logger");
    changes = await executeTx(client, keypair, tx);
    const allCaps = findCreated(changes, "Capability");
    const loggerCapId = allCaps.find((id) => id !== adminCapId)!;
    console.log(`  LoggerCap: ${loggerCapId}`);

    // ===== Phase 5: Create ACB =====
    console.log("\nPhase 5: Creating ACB...");

    tx = new Transaction();
    const acb = h.acbCreate(tx, federationId, trailId, [
        { name: "catch_logger", properties: [["catch_logging", "Cod"]] },
    ]);
    h.acbShare(tx, acb);
    changes = await executeTx(client, keypair, tx);
    const acbId = findCreated(changes, "AccessControllerBridge")[0];
    console.log(`  ACB: ${acbId}`);

    // ===== Phase 6: Deposit capability =====
    console.log("\nPhase 6: Depositing capability...");

    tx = new Transaction();
    h.acbDeposit(tx, acbId, federationId, "catch_logger", loggerCapId);
    await executeTx(client, keypair, tx);
    console.log("  Capability deposited");

    // ===== Phase 7: Accredit attester =====
    console.log("\nPhase 7: Accrediting attester...");

    tx = new Transaction();
    h.fedAccreditToAttest(tx, federationId, accreditCapId, address, [
        { name: "catch_logging", values: ["Cod", "Haddock"], allowAny: false },
    ]);
    await executeTx(client, keypair, tx);
    console.log("  Attester accredited for catch_logging = [Cod, Haddock]");

    console.log("\n=== Setup Complete ===");
    console.log(`Federation:  ${federationId}`);
    console.log(`AuditTrail:  ${trailId}`);
    console.log(`ACB:         ${acbId}`);
    console.log(`AdminCap:    ${adminCapId}`);
}

main().catch((e) => {
    console.error(e);
    process.exit(1);
});
