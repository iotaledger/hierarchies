// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Full initialization of the Access Controller Bridge.
//!
//! Uses `PtbHelper` to encapsulate all raw PTB construction.
//!
//! Run:
//!   IOTA_HIERARCHIES_PKG_ID=0x... IOTA_AUDIT_TRAIL_PKG_ID=0x... \
//!   IOTA_TF_COMPONENTS_PKG_ID=0x... IOTA_ACB_PKG_ID=0x... \
//!   cargo run --example 01_full_initialization

use acb_examples::*;
use anyhow::Context;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::ObjectArg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let h = PtbHelper::new()?;
    let (client, signer) = get_client_and_signer().await?;
    let sender = signer.get_address().await?;

    // ===== Phase 1: Create federation + property =====
    println!("Phase 1: Creating federation...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    h.new_federation(&mut ptb);
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let federation_id = find_created(&changes, "Federation")[0];
    let root_cap_id = find_created(&changes, "RootAuthorityCap")[0];
    let accredit_cap_id = find_created(&changes, "AccreditCap")[0];
    println!("  Federation: {federation_id}");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let fed = ptb.obj(shared_arg(&client, federation_id, true).await?)?;
    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, root_cap_id).await?))?;
    h.fed_add_property(&mut ptb, fed, cap, "catch_logging", vec!["Cod", "Haddock"], false)?;
    execute_ptb(&client, &signer, ptb).await?;
    println!("  catch_logging property added");

    // ===== Phase 2: Create audit trail =====
    println!("\nPhase 2: Creating audit trail...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    h.trail_create(&mut ptb, "ACB Test Trail", sender)?;
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let trail_id = find_created(&changes, "AuditTrail")[0];
    let admin_cap_id = find_created(&changes, "Capability")[0];
    println!("  AuditTrail: {trail_id}");

    // ===== Phase 3: Create role =====
    println!("\nPhase 3: Creating role...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let ta = ptb.obj(shared_arg(&client, trail_id, true).await?)?;
    let ca = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, admin_cap_id).await?))?;
    h.trail_create_role(&mut ptb, ta, ca, "catch_logger", vec!["add_record"])?;
    execute_ptb(&client, &signer, ptb).await?;
    println!("  catch_logger role created");

    // ===== Phase 4: Mint capability =====
    println!("\nPhase 4: Minting capability...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let ta = ptb.obj(shared_arg(&client, trail_id, true).await?)?;
    let ca = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, admin_cap_id).await?))?;
    h.trail_mint_capability(&mut ptb, ta, ca, "catch_logger")?;
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let logger_cap_id = find_created(&changes, "Capability")
        .into_iter().find(|id| *id != admin_cap_id).context("no new cap")?;
    println!("  LoggerCap: {logger_cap_id}");

    // ===== Phase 5: Create ACB =====
    println!("\nPhase 5: Creating ACB...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let fa = ptb.obj(shared_arg(&client, federation_id, false).await?)?;
    let acb = h.acb_create(&mut ptb, fa, trail_id, vec![
        ("catch_logger", vec![("catch_logging", "Cod")]),
    ])?;
    h.share_acb(&mut ptb, acb);
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let acb_id = find_created(&changes, "AccessControllerBridge")[0];
    println!("  ACB: {acb_id}");

    // ===== Phase 6: Deposit capability =====
    println!("\nPhase 6: Depositing capability...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let aa = ptb.obj(shared_arg(&client, acb_id, true).await?)?;
    let fa = ptb.obj(shared_arg(&client, federation_id, false).await?)?;
    let ca = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, logger_cap_id).await?))?;
    h.acb_deposit(&mut ptb, aa, fa, "catch_logger", ca)?;
    execute_ptb(&client, &signer, ptb).await?;
    println!("  Capability deposited");

    // ===== Phase 7: Accredit attester =====
    println!("\nPhase 7: Accrediting attester...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let fa = ptb.obj(shared_arg(&client, federation_id, true).await?)?;
    let ac = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_ref(&client, accredit_cap_id).await?))?;
    h.fed_accredit_to_attest(&mut ptb, fa, ac, sender, vec![
        ("catch_logging", vec!["Cod", "Haddock"], false),
    ])?;
    execute_ptb(&client, &signer, ptb).await?;
    println!("  Attester accredited for catch_logging = [Cod, Haddock]");

    println!("\n=== Setup Complete ===");
    println!("Federation:  {federation_id}");
    println!("AuditTrail:  {trail_id}");
    println!("ACB:         {acb_id}");
    println!("AdminCap:    {admin_cap_id}");

    Ok(())
}
