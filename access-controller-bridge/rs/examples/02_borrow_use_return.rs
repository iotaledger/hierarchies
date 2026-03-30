// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Demonstrates the Borrow–Use–Return flow in a single PTB.
//!
//! Self-contained: sets up everything, then performs the three-step flow.
//!
//! Run:
//!   IOTA_HIERARCHIES_PKG_ID=0x... IOTA_AUDIT_TRAIL_PKG_ID=0x... \
//!   IOTA_TF_COMPONENTS_PKG_ID=0x... IOTA_ACB_PKG_ID=0x... \
//!   cargo run --example 02_borrow_use_return

use acb_examples::*;
use anyhow::Context;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::ObjectArg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let h = PtbHelper::new()?;
    let (client, signer) = get_client_and_signer().await?;
    let sender = signer.get_address().await?;

    // ===== Setup (condensed) =====
    println!("Setting up...\n");

    // Federation + property
    let mut ptb = ProgrammableTransactionBuilder::new();
    h.new_federation(&mut ptb);
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let federation_id = find_created(&changes, "Federation")[0];
    let root_cap_id = find_created(&changes, "RootAuthorityCap")[0];
    let accredit_cap_id = find_created(&changes, "AccreditCap")[0];

    let mut ptb = ProgrammableTransactionBuilder::new();
    let fa = ptb.obj(shared_arg(&client, federation_id, true).await?)?;
    let ca = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, root_cap_id).await?))?;
    h.fed_add_property(&mut ptb, fa, ca, "catch_logging", vec!["Cod", "Haddock"], false)?;
    execute_ptb(&client, &signer, ptb).await?;
    println!("  Federation:  {federation_id}");

    // Trail + role + cap
    let mut ptb = ProgrammableTransactionBuilder::new();
    h.trail_create(&mut ptb, "ACB Test Trail", sender)?;
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let trail_id = find_created(&changes, "AuditTrail")[0];
    let admin_cap_id = find_created(&changes, "Capability")[0];

    let mut ptb = ProgrammableTransactionBuilder::new();
    let ta = ptb.obj(shared_arg(&client, trail_id, true).await?)?;
    let ca = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, admin_cap_id).await?))?;
    h.trail_create_role(&mut ptb, ta, ca, "catch_logger", vec!["add_record"])?;
    execute_ptb(&client, &signer, ptb).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();
    let ta = ptb.obj(shared_arg(&client, trail_id, true).await?)?;
    let ca = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, admin_cap_id).await?))?;
    h.trail_mint_capability(&mut ptb, ta, ca, "catch_logger")?;
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let logger_cap_id = find_created(&changes, "Capability")
        .into_iter().find(|id| *id != admin_cap_id).context("no cap")?;
    println!("  AuditTrail:  {trail_id}");

    // ACB + deposit
    let mut ptb = ProgrammableTransactionBuilder::new();
    let fa = ptb.obj(shared_arg(&client, federation_id, false).await?)?;
    let acb = h.acb_create(&mut ptb, fa, trail_id, vec![("catch_logger", vec![("catch_logging", "Cod")])])?;
    h.share_acb(&mut ptb, acb);
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let acb_id = find_created(&changes, "AccessControllerBridge")[0];

    let mut ptb = ProgrammableTransactionBuilder::new();
    let aa = ptb.obj(shared_arg(&client, acb_id, true).await?)?;
    let fa = ptb.obj(shared_arg(&client, federation_id, false).await?)?;
    let ca = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, logger_cap_id).await?))?;
    h.acb_deposit(&mut ptb, aa, fa, "catch_logger", ca)?;
    execute_ptb(&client, &signer, ptb).await?;
    println!("  ACB:         {acb_id}");

    // Accredit self
    let mut ptb = ProgrammableTransactionBuilder::new();
    let fa = ptb.obj(shared_arg(&client, federation_id, true).await?)?;
    let ac = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, accredit_cap_id).await?))?;
    h.fed_accredit_to_attest(&mut ptb, fa, ac, sender, vec![
        ("catch_logging", vec!["Cod", "Haddock"], false),
    ])?;
    execute_ptb(&client, &signer, ptb).await?;
    println!("  Attester accredited\n");

    // ==========================================================
    // THE CORE FLOW: Borrow–Use–Return in a single PTB
    // ==========================================================
    println!("=== Borrow–Use–Return PTB ===\n");

    let mut ptb = ProgrammableTransactionBuilder::new();

    // Step 1: borrow()
    let acb_arg = ptb.obj(shared_arg(&client, acb_id, true).await?)?;
    let fed_arg = ptb.obj(shared_arg(&client, federation_id, false).await?)?;
    let (cap, receipt) = h.acb_borrow(
        &mut ptb, acb_arg, fed_arg,
        "catch_logger",
    )?;
    println!("  Step 1: borrow(RoleName(\"catch_logger\"))");

    // Step 2: add_record()
    let trail_arg = ptb.obj(shared_arg(&client, trail_id, true).await?)?;
    h.trail_add_record(&mut ptb, trail_arg, cap, "Cod catch logged via ACB")?;
    println!("  Step 2: add_record(\"Cod catch logged via ACB\")");

    // Step 3: return_cap()
    let acb_arg2 = ptb.obj(shared_arg(&client, acb_id, true).await?)?;
    h.acb_return(&mut ptb, acb_arg2, cap, receipt);
    println!("  Step 3: return_cap()");

    // Execute
    println!("\n  Executing PTB...");
    execute_ptb(&client, &signer, ptb).await?;
    println!("\n  SUCCESS! Record added to audit trail via ACB.");

    Ok(())
}
