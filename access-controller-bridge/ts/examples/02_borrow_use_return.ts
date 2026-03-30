// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Demonstrates the Borrow-Use-Return flow in a single PTB.
//
// Self-contained: sets up everything, then performs the three-step flow.
//
// Run:
//   IOTA_HIERARCHIES_PKG_ID=0x... IOTA_AUDIT_TRAIL_PKG_ID=0x... \
//   IOTA_TF_COMPONENTS_PKG_ID=0x... IOTA_ACB_PKG_ID=0x... \
//   npx ts-node 02_borrow_use_return.ts

import { Transaction } from "@iota/iota-sdk/transactions";
import { PtbHelper, getClientAndSigner, executeTx, findCreated } from "./utils";

async function main() {
    const h = new PtbHelper();
    const { client, keypair, address } = await getClientAndSigner();

    // ===== Setup (condensed) =====
    console.log("Setting up...\n");

    // Federation + property
    let tx = new Transaction();
    h.newFederation(tx);
    let changes = await executeTx(client, keypair, tx);
    const federationId = findCreated(changes, "Federation")[0];
    const rootCapId = findCreated(changes, "RootAuthorityCap")[0];
    const accreditCapId = findCreated(changes, "AccreditCap")[0];

    tx = new Transaction();
    h.fedAddProperty(tx, federationId, rootCapId, "catch_logging", ["Cod", "Haddock"], false);
    await executeTx(client, keypair, tx);
    console.log(`  Federation:  ${federationId}`);

    // Trail + role + cap
    tx = new Transaction();
    h.trailCreate(tx, "ACB Test Trail", address);
    changes = await executeTx(client, keypair, tx);
    const trailId = findCreated(changes, "AuditTrail")[0];
    const adminCapId = findCreated(changes, "Capability")[0];

    tx = new Transaction();
    h.trailCreateRole(tx, trailId, adminCapId, "catch_logger", ["add_record"]);
    await executeTx(client, keypair, tx);

    tx = new Transaction();
    h.trailMintCapability(tx, trailId, adminCapId, "catch_logger");
    changes = await executeTx(client, keypair, tx);
    const loggerCapId = findCreated(changes, "Capability").find((id) => id !== adminCapId)!;
    console.log(`  AuditTrail:  ${trailId}`);

    // ACB + deposit
    tx = new Transaction();
    const acb = h.acbCreate(tx, federationId, trailId, [
        { name: "catch_logger", properties: [["catch_logging", "Cod"]] },
    ]);
    h.acbShare(tx, acb);
    changes = await executeTx(client, keypair, tx);
    const acbId = findCreated(changes, "AccessControllerBridge")[0];

    tx = new Transaction();
    h.acbDeposit(tx, acbId, federationId, "catch_logger", loggerCapId);
    await executeTx(client, keypair, tx);
    console.log(`  ACB:         ${acbId}`);

    // Accredit self
    tx = new Transaction();
    h.fedAccreditToAttest(tx, federationId, accreditCapId, address, [
        { name: "catch_logging", values: ["Cod", "Haddock"], allowAny: false },
    ]);
    await executeTx(client, keypair, tx);
    console.log("  Attester accredited\n");

    // ==========================================================
    // THE CORE FLOW: Borrow-Use-Return in a single PTB
    // ==========================================================
    console.log("=== Borrow-Use-Return PTB ===\n");

    tx = new Transaction();

    // Step 1: borrow()
    const { cap, receipt } = h.acbBorrow(tx, acbId, federationId, "catch_logger");
    console.log('  Step 1: borrow(RoleName("catch_logger"))');

    // Step 2: add_record()
    h.trailAddRecord(tx, trailId, cap, "Cod catch logged via ACB");
    console.log('  Step 2: add_record("Cod catch logged via ACB")');

    // Step 3: return_cap()
    h.acbReturn(tx, acbId, cap, receipt);
    console.log("  Step 3: return_cap()");

    // Execute
    console.log("\n  Executing PTB...");
    await executeTx(client, keypair, tx);
    console.log("\n  SUCCESS! Record added to audit trail via ACB.");
}

main().catch((e) => {
    console.error(e);
    process.exit(1);
});
