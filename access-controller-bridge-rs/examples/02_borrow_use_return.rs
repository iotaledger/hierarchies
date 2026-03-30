// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Demonstrates the complete Borrow–Use–Return flow in a single PTB.
//!
//! This example is self-contained: it sets up the federation, trail, ACB,
//! accredits the sender, then performs the three-step borrow-use-return.
//!
//! Run:
//!   IOTA_HIERARCHIES_PKG_ID=0x... IOTA_AUDIT_TRAIL_PKG_ID=0x... \
//!   IOTA_TF_COMPONENTS_PKG_ID=0x... IOTA_ACB_PKG_ID=0x... \
//!   cargo run --example 02_borrow_use_return

use acb_examples::*;
use anyhow::Context;
use iota_interaction::ident_str;
use iota_interaction::types::Identifier;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::{Argument, Command, ObjectArg};
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hier_pkg = hierarchies_pkg()?;
    let at_pkg = audit_trail_pkg()?;
    let acb_pkg_id = acb_pkg()?;
    let tf_pkg = tf_components_pkg()?;

    let client = get_iota_client().await?;
    let signer = get_funded_signer().await?;
    let sender = signer.get_address().await?;

    let data_tag = type_tag(at_pkg, "record", "Data");
    let marker_tag = iota_interaction::types::TypeTag::from_str("bool")?;

    // ==========================================================
    // Setup (same as 01_full_initialization, condensed)
    // ==========================================================
    println!("Setting up federation, trail, ACB...\n");

    // -- Create federation --
    let mut ptb = ProgrammableTransactionBuilder::new();
    ptb.programmable_move_call(
        hier_pkg, ident_str!("main").into(), ident_str!("new_federation").into(), vec![], vec![],
    );
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let federation_id = find_created(&changes, "Federation")[0];
    let root_cap_id = find_created(&changes, "RootAuthorityCap")[0];
    let accredit_cap_id = find_created(&changes, "AccreditCap")[0];
    println!("  Federation:  {federation_id}");

    // -- Add catch_logging property --
    let mut ptb = ProgrammableTransactionBuilder::new();
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, true).await?)?;
    let cap_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, root_cap_id).await?))?;
    let prop_name = ptb_property_name(&mut ptb, hier_pkg, "catch_logging")?;
    let val1 = ptb_property_value_string(&mut ptb, hier_pkg, "Cod")?;
    let val2 = ptb_property_value_string(&mut ptb, hier_pkg, "Haddock")?;
    let value_tag = type_tag(hier_pkg, "property_value", "PropertyValue");
    let vals_vec = ptb.command(Command::MakeMoveVec(Some(value_tag.clone().into()), vec![val1, val2]));
    let allowed_set = ptb.programmable_move_call(
        hier_pkg, ident_str!("utils").into(), ident_str!("create_vec_set").into(),
        vec![value_tag], vec![vals_vec],
    );
    let shape_none = option_none(&mut ptb, type_tag(hier_pkg, "property_shape", "PropertyShape"))?;
    let allow_any = ptb.pure(false)?;
    let property = ptb.programmable_move_call(
        hier_pkg, ident_str!("property").into(), ident_str!("new_property").into(),
        vec![], vec![prop_name, allowed_set, allow_any, shape_none],
    );
    ptb.programmable_move_call(
        hier_pkg, ident_str!("main").into(), ident_str!("add_property").into(),
        vec![], vec![fed_arg, cap_arg, property],
    );
    execute_ptb(&client, &signer, ptb).await?;

    // -- Create audit trail --
    let mut ptb = ProgrammableTransactionBuilder::new();
    let ir_tag = iota_interaction::types::TypeTag::from_str(
        &format!("{at_pkg}::record::InitialRecord<{at_pkg}::record::Data>")
    )?;
    let initial_record = option_none(&mut ptb, ir_tag)?;
    let window_none = ptb.programmable_move_call(
        at_pkg, ident_str!("locking").into(), ident_str!("window_none").into(), vec![], vec![],
    );
    let tl1 = ptb.programmable_move_call(
        tf_pkg, ident_str!("timelock").into(), ident_str!("none").into(), vec![], vec![],
    );
    let tl2 = ptb.programmable_move_call(
        tf_pkg, ident_str!("timelock").into(), ident_str!("none").into(), vec![], vec![],
    );
    let locking_config = ptb.programmable_move_call(
        at_pkg, ident_str!("locking").into(), ident_str!("new").into(),
        vec![], vec![window_none, tl1, tl2],
    );
    let name_arg = ptb.pure("ACB Test Trail".to_string())?;
    let desc_none = ptb.pure(Option::<String>::None)?;
    let metadata = ptb.programmable_move_call(
        at_pkg, ident_str!("main").into(), ident_str!("new_trail_metadata").into(),
        vec![], vec![name_arg, desc_none],
    );
    let md_tag = iota_interaction::types::TypeTag::from_str(
        &format!("{at_pkg}::main::ImmutableMetadata")
    )?;
    let trail_metadata = option_some(&mut ptb, md_tag, metadata)?;
    let updatable_none = ptb.pure(Option::<String>::None)?;
    let tags = ptb.pure(Vec::<String>::new())?;
    let clock = clock_arg(&mut ptb);
    let result = ptb.programmable_move_call(
        at_pkg, ident_str!("main").into(), ident_str!("create").into(),
        vec![data_tag.clone()], vec![initial_record, locking_config, trail_metadata, updatable_none, tags, clock],
    );
    let admin_cap = match result { Argument::Result(idx) => Argument::NestedResult(idx, 0), _ => unreachable!() };
    ptb.transfer_arg(sender, admin_cap);
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let trail_id = find_created(&changes, "AuditTrail")[0];
    let admin_cap_id = find_created(&changes, "Capability")[0];
    println!("  AuditTrail:  {trail_id}");

    // -- Create role + mint cap --
    let mut ptb = ProgrammableTransactionBuilder::new();
    let trail_arg = ptb.obj(shared_obj_arg(&client, trail_id, true).await?)?;
    let cap_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, admin_cap_id).await?))?;
    let role = ptb.pure("catch_logger".to_string())?;
    let add_perm = ptb.programmable_move_call(
        at_pkg, ident_str!("permission").into(), ident_str!("add_record").into(), vec![], vec![],
    );
    let perm_tag = type_tag(at_pkg, "permission", "Permission");
    let perms_vec = ptb.command(Command::MakeMoveVec(Some(perm_tag.into()), vec![add_perm]));
    let perm_set = ptb.programmable_move_call(
        at_pkg, ident_str!("permission").into(), ident_str!("from_vec").into(), vec![], vec![perms_vec],
    );
    let rt_tag = iota_interaction::types::TypeTag::from_str(
        &format!("{at_pkg}::record_tags::RoleTags")
    )?;
    let rt_none = option_none(&mut ptb, rt_tag)?;
    let clock = clock_arg(&mut ptb);
    ptb.programmable_move_call(
        at_pkg, ident_str!("main").into(), ident_str!("create_role").into(),
        vec![data_tag.clone()], vec![trail_arg, cap_arg, role, perm_set, rt_none, clock],
    );
    execute_ptb(&client, &signer, ptb).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();
    let trail_arg = ptb.obj(shared_obj_arg(&client, trail_id, true).await?)?;
    let cap_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, admin_cap_id).await?))?;
    let role = ptb.pure("catch_logger".to_string())?;
    let none_addr = ptb.pure(Option::<Vec<u8>>::None)?;
    let none_from = ptb.pure(Option::<u64>::None)?;
    let none_until = ptb.pure(Option::<u64>::None)?;
    let clock = clock_arg(&mut ptb);
    ptb.programmable_move_call(
        at_pkg, ident_str!("main").into(), ident_str!("new_capability").into(),
        vec![data_tag.clone()], vec![trail_arg, cap_arg, role, none_addr, none_from, none_until, clock],
    );
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let logger_cap_id = find_created(&changes, "Capability")
        .into_iter().find(|id| *id != admin_cap_id).context("no new cap")?;

    // -- Create ACB + deposit --
    let mut ptb = ProgrammableTransactionBuilder::new();
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, false).await?)?;
    let target_id = ptb.pure(trail_id)?;
    let type_name = ptb.pure("catch_logger".to_string())?;
    let pn = ptb_property_name(&mut ptb, hier_pkg, "catch_logging")?;
    let config = ptb_new_cap_type_config(&mut ptb, hier_pkg, acb_pkg_id, vec![pn]);
    let config_tag = type_tag(acb_pkg_id, "bridge", "CapabilityTypeConfig");
    let configs_map = ptb_vec_map_from_keys_values(
        &mut ptb, hier_pkg, string_type_tag(), config_tag, vec![type_name], vec![config],
    );
    let acb_result = ptb.programmable_move_call(
        acb_pkg_id, ident_str!("bridge").into(), ident_str!("create").into(),
        vec![marker_tag.clone()], vec![fed_arg, target_id, configs_map],
    );
    let iota_fw = iota_interaction::types::base_types::ObjectID::from_hex_literal("0x2").unwrap();
    let acb_tt = iota_interaction::types::TypeTag::from_str(
        &format!("{acb_pkg_id}::bridge::AccessControllerBridge<bool>")
    )?;
    ptb.programmable_move_call(
        iota_fw, Identifier::new("transfer").unwrap(), Identifier::new("public_share_object").unwrap(),
        vec![acb_tt], vec![acb_result],
    );
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let acb_id = find_created(&changes, "AccessControllerBridge")[0];
    println!("  ACB:         {acb_id}");

    // Deposit cap
    let mut ptb = ProgrammableTransactionBuilder::new();
    let acb_arg = ptb.obj(shared_obj_arg(&client, acb_id, true).await?)?;
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, false).await?)?;
    let ct = ptb.pure("catch_logger".to_string())?;
    let cap_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, logger_cap_id).await?))?;
    ptb.programmable_move_call(
        acb_pkg_id, ident_str!("bridge").into(), ident_str!("deposit_capability").into(),
        vec![marker_tag.clone()], vec![acb_arg, fed_arg, ct, cap_arg],
    );
    execute_ptb(&client, &signer, ptb).await?;

    // -- Accredit self --
    let mut ptb = ProgrammableTransactionBuilder::new();
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, true).await?)?;
    let accr_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, accredit_cap_id).await?))?;
    let receiver_id = ptb.pure(sender)?;
    let pn = ptb_property_name(&mut ptb, hier_pkg, "catch_logging")?;
    let v1 = ptb_property_value_string(&mut ptb, hier_pkg, "Cod")?;
    let v2 = ptb_property_value_string(&mut ptb, hier_pkg, "Haddock")?;
    let value_tag = type_tag(hier_pkg, "property_value", "PropertyValue");
    let vv = ptb.command(Command::MakeMoveVec(Some(value_tag.clone().into()), vec![v1, v2]));
    let aset = ptb.programmable_move_call(
        hier_pkg, ident_str!("utils").into(), ident_str!("create_vec_set").into(),
        vec![value_tag], vec![vv],
    );
    let aa = ptb.pure(false)?;
    let sn = option_none(&mut ptb, type_tag(hier_pkg, "property_shape", "PropertyShape"))?;
    let fp = ptb.programmable_move_call(
        hier_pkg, ident_str!("property").into(), ident_str!("new_property").into(),
        vec![], vec![pn, aset, aa, sn],
    );
    let pt = type_tag(hier_pkg, "property", "FederationProperty");
    let pv = ptb.command(Command::MakeMoveVec(Some(pt.into()), vec![fp]));
    let clock = clock_arg(&mut ptb);
    ptb.programmable_move_call(
        hier_pkg, ident_str!("main").into(), ident_str!("create_accreditation_to_attest").into(),
        vec![], vec![fed_arg, accr_arg, receiver_id, pv, clock],
    );
    execute_ptb(&client, &signer, ptb).await?;
    println!("  Attester accredited\n");

    // ==========================================================
    // THE CORE FLOW: Borrow–Use–Return in a single PTB
    // ==========================================================
    println!("=== Borrow–Use–Return PTB ===\n");

    let mut ptb = ProgrammableTransactionBuilder::new();

    // ----- Step 1: bridge::borrow() -----
    println!("  Step 1: borrow(catch_logger, {{catch_logging: Cod}})");
    let acb_arg = ptb.obj(shared_obj_arg(&client, acb_id, true).await?)?;
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, false).await?)?;
    let cap_type = ptb.pure("catch_logger".to_string())?;

    let pn = ptb_property_name(&mut ptb, hier_pkg, "catch_logging")?;
    let pv = ptb_property_value_string(&mut ptb, hier_pkg, "Cod")?;
    let name_tag = type_tag(hier_pkg, "property_name", "PropertyName");
    let val_tag = type_tag(hier_pkg, "property_value", "PropertyValue");
    let prop_map = ptb_vec_map_from_keys_values(
        &mut ptb, hier_pkg, name_tag, val_tag, vec![pn], vec![pv],
    );
    let clock = clock_arg(&mut ptb);

    let borrow_result = ptb.programmable_move_call(
        acb_pkg_id, ident_str!("bridge").into(), ident_str!("borrow").into(),
        vec![marker_tag.clone()], vec![acb_arg, fed_arg, cap_type, prop_map, clock],
    );
    let cap = match borrow_result { Argument::Result(idx) => Argument::NestedResult(idx, 0), _ => unreachable!() };
    let receipt = match borrow_result { Argument::Result(idx) => Argument::NestedResult(idx, 1), _ => unreachable!() };

    // ----- Step 2: audit_trail::add_record(&cap) -----
    println!("  Step 2: add_record(\"Cod catch logged via ACB\")");
    let trail_arg = ptb.obj(shared_obj_arg(&client, trail_id, true).await?)?;
    let text = ptb.pure("Cod catch logged via ACB".to_string())?;
    let record_data = ptb.programmable_move_call(
        at_pkg, ident_str!("record").into(), ident_str!("new_text").into(),
        vec![], vec![text],
    );
    let md_none = ptb.pure(Option::<String>::None)?;
    let tag_none = ptb.pure(Option::<String>::None)?;
    let clock2 = clock_arg(&mut ptb);
    ptb.programmable_move_call(
        at_pkg, ident_str!("main").into(), ident_str!("add_record").into(),
        vec![data_tag], vec![trail_arg, cap, record_data, md_none, tag_none, clock2],
    );

    // ----- Step 3: bridge::return_cap() -----
    println!("  Step 3: return_cap()");
    let acb_arg2 = ptb.obj(shared_obj_arg(&client, acb_id, true).await?)?;
    let clock3 = clock_arg(&mut ptb);
    ptb.programmable_move_call(
        acb_pkg_id, ident_str!("bridge").into(), ident_str!("return_cap").into(),
        vec![marker_tag], vec![acb_arg2, cap, receipt, clock3],
    );

    println!("\n  Executing PTB...");
    execute_ptb(&client, &signer, ptb).await?;

    println!("\n  SUCCESS! Record added to audit trail via ACB.");
    println!("  The Capability was borrowed, used, and returned in a single atomic transaction.");

    Ok(())
}

// ===== Option helpers =====

fn option_none(
    ptb: &mut ProgrammableTransactionBuilder,
    inner_tag: iota_interaction::types::TypeTag,
) -> anyhow::Result<Argument> {
    Ok(ptb.programmable_move_call(
        iota_interaction::types::base_types::ObjectID::from_hex_literal("0x1").unwrap(),
        ident_str!("option").into(),
        ident_str!("none").into(),
        vec![inner_tag],
        vec![],
    ))
}

fn option_some(
    ptb: &mut ProgrammableTransactionBuilder,
    inner_tag: iota_interaction::types::TypeTag,
    value: Argument,
) -> anyhow::Result<Argument> {
    Ok(ptb.programmable_move_call(
        iota_interaction::types::base_types::ObjectID::from_hex_literal("0x1").unwrap(),
        ident_str!("option").into(),
        ident_str!("some").into(),
        vec![inner_tag],
        vec![value],
    ))
}
