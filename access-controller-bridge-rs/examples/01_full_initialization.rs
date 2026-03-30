// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Full initialization of the Access Controller Bridge.
//!
//! Demonstrates the complete setup flow:
//!   1. Create federation + add properties
//!   2. Create audit trail
//!   3. Create roles on the trail
//!   4. Mint bearer Capabilities
//!   5. Create the AccessControllerBridge
//!   6. Deposit Capabilities into the ACB
//!   7. Accredit an attester in the federation
//!
//! Run:
//!   IOTA_HIERARCHIES_PKG_ID=0x... IOTA_AUDIT_TRAIL_PKG_ID=0x... \
//!   IOTA_TF_COMPONENTS_PKG_ID=0x... IOTA_ACB_PKG_ID=0x... \
//!   cargo run --example 01_full_initialization

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
    let _tf_pkg = tf_components_pkg()?;

    let client = get_iota_client().await?;
    let signer = get_funded_signer().await?;
    let sender = signer.get_address().await?;

    // =========================================================
    // Phase 1: Create federation + add properties
    // =========================================================
    println!("Phase 1: Creating federation...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    ptb.programmable_move_call(
        hier_pkg,
        ident_str!("main").into(),
        ident_str!("new_federation").into(),
        vec![],
        vec![],
    );
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let federation_id = find_created(&changes, "Federation")[0];
    let root_cap_id = find_created(&changes, "RootAuthorityCap")[0];
    let accredit_cap_id = find_created(&changes, "AccreditCap")[0];
    println!("  Federation:   {federation_id}");
    println!("  RootCap:      {root_cap_id}");
    println!("  AccreditCap:  {accredit_cap_id}");

    // Add catch_logging property
    println!("  Adding catch_logging property...");
    let mut ptb = ProgrammableTransactionBuilder::new();
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, true).await?)?;
    let cap_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, root_cap_id).await?))?;

    // Build FederationProperty: new_property(name, allowed_values, allow_any, shape)
    let prop_name = ptb_property_name(&mut ptb, hier_pkg, "catch_logging")?;
    let val_cod = ptb_property_value_string(&mut ptb, hier_pkg, "Cod")?;
    let val_haddock = ptb_property_value_string(&mut ptb, hier_pkg, "Haddock")?;

    let value_tag = type_tag(hier_pkg, "property_value", "PropertyValue");
    let vals_vec = ptb.command(Command::MakeMoveVec(Some(value_tag.clone().into()), vec![val_cod, val_haddock]));
    let allowed_set = ptb.programmable_move_call(
        hier_pkg,
        ident_str!("utils").into(),
        ident_str!("create_vec_set").into(),
        vec![value_tag],
        vec![vals_vec],
    );

    let shape_tag = type_tag(hier_pkg, "property_shape", "PropertyShape");
    let shape_none = option_none(&mut ptb, shape_tag)?;
    let allow_any = ptb.pure(false)?;

    let property = ptb.programmable_move_call(
        hier_pkg,
        ident_str!("property").into(),
        ident_str!("new_property").into(),
        vec![],
        vec![prop_name, allowed_set, allow_any, shape_none],
    );
    ptb.programmable_move_call(
        hier_pkg,
        ident_str!("main").into(),
        ident_str!("add_property").into(),
        vec![],
        vec![fed_arg, cap_arg, property],
    );
    execute_ptb(&client, &signer, ptb).await?;
    println!("  catch_logging property added");

    // =========================================================
    // Phase 2: Create audit trail
    // =========================================================
    println!("\nPhase 2: Creating audit trail...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let data_tag = type_tag(at_pkg, "record", "Data");

    // initial_record: Option<InitialRecord<Data>> = none
    let ir_tag_str = format!("{at_pkg}::record::InitialRecord<{at_pkg}::record::Data>");
    let ir_tag = iota_interaction::types::TypeTag::from_str(&ir_tag_str)?;
    let initial_record = option_none(&mut ptb, ir_tag)?;

    // locking_config: new(window_none(), timelock::none(), timelock::none())
    let window_none = ptb.programmable_move_call(
        at_pkg, ident_str!("locking").into(), ident_str!("window_none").into(), vec![], vec![],
    );
    let tf_pkg = tf_components_pkg()?;
    let tl_none1 = ptb.programmable_move_call(
        tf_pkg, ident_str!("timelock").into(), ident_str!("none").into(), vec![], vec![],
    );
    let tl_none2 = ptb.programmable_move_call(
        tf_pkg, ident_str!("timelock").into(), ident_str!("none").into(), vec![], vec![],
    );
    let locking_config = ptb.programmable_move_call(
        at_pkg, ident_str!("locking").into(), ident_str!("new").into(),
        vec![], vec![window_none, tl_none1, tl_none2],
    );

    // trail_metadata: some(new_trail_metadata("ACB Test Trail", none))
    let name_arg = ptb.pure("ACB Test Trail".to_string())?;
    let desc_none = option_none_string(&mut ptb)?;
    let metadata = ptb.programmable_move_call(
        at_pkg,
        ident_str!("main").into(),
        ident_str!("new_trail_metadata").into(),
        vec![],
        vec![name_arg, desc_none],
    );
    let md_tag_str = format!("{at_pkg}::main::ImmutableMetadata");
    let md_tag = iota_interaction::types::TypeTag::from_str(&md_tag_str)?;
    let trail_metadata = option_some(&mut ptb, md_tag, metadata)?;

    // updatable_metadata: none
    let updatable_none = option_none_string(&mut ptb)?;

    // tags: empty vector<String>
    let tags = ptb.pure(Vec::<String>::new())?;

    let clock = clock_arg(&mut ptb);

    let result = ptb.programmable_move_call(
        at_pkg,
        ident_str!("main").into(),
        ident_str!("create").into(),
        vec![data_tag.clone()],
        vec![initial_record, locking_config, trail_metadata, updatable_none, tags, clock],
    );

    // Extract and transfer the admin capability (result index 0)
    let admin_cap = match result {
        Argument::Result(idx) => Argument::NestedResult(idx, 0),
        _ => unreachable!(),
    };
    ptb.transfer_arg(sender, admin_cap);

    let changes = execute_ptb(&client, &signer, ptb).await?;
    let trail_id = find_created(&changes, "AuditTrail")[0];
    let admin_cap_id = find_created(&changes, "Capability")[0];
    println!("  AuditTrail: {trail_id}");
    println!("  AdminCap:   {admin_cap_id}");

    // =========================================================
    // Phase 3: Create roles on the trail
    // =========================================================
    println!("\nPhase 3: Creating roles...");

    // Create "catch_logger" role with AddRecord permission
    let mut ptb = ProgrammableTransactionBuilder::new();
    let trail_arg = ptb.obj(shared_obj_arg(&client, trail_id, true).await?)?;
    let cap_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, admin_cap_id).await?))?;
    let role = ptb.pure("catch_logger".to_string())?;

    // Build permission set: from_vec(vec[add_record()])
    let add_record_perm = ptb.programmable_move_call(
        at_pkg,
        ident_str!("permission").into(),
        ident_str!("add_record").into(),
        vec![],
        vec![],
    );
    let perm_tag = type_tag(at_pkg, "permission", "Permission");
    let perms_vec = ptb.command(Command::MakeMoveVec(Some(perm_tag.clone().into()), vec![add_record_perm]));
    let perm_set = ptb.programmable_move_call(
        at_pkg,
        ident_str!("permission").into(),
        ident_str!("from_vec").into(),
        vec![],
        vec![perms_vec],
    );

    let role_tags_tag_str = format!("{at_pkg}::record_tags::RoleTags");
    let role_tags_tag = iota_interaction::types::TypeTag::from_str(&role_tags_tag_str)?;
    let role_tags_none = option_none(&mut ptb, role_tags_tag)?;
    let clock = clock_arg(&mut ptb);

    ptb.programmable_move_call(
        at_pkg,
        ident_str!("main").into(),
        ident_str!("create_role").into(),
        vec![data_tag.clone()],
        vec![trail_arg, cap_arg, role, perm_set, role_tags_none, clock],
    );
    execute_ptb(&client, &signer, ptb).await?;
    println!("  catch_logger role created");

    // =========================================================
    // Phase 4: Mint bearer Capability for catch_logger
    // =========================================================
    println!("\nPhase 4: Minting capability...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let trail_arg = ptb.obj(shared_obj_arg(&client, trail_id, true).await?)?;
    let cap_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, admin_cap_id).await?))?;
    let role = ptb.pure("catch_logger".to_string())?;
    let issued_to_none = ptb.pure(Option::<Vec<u8>>::None)?;
    let valid_from_none = ptb.pure(Option::<u64>::None)?;
    let valid_until_none = ptb.pure(Option::<u64>::None)?;
    let clock = clock_arg(&mut ptb);

    ptb.programmable_move_call(
        at_pkg,
        ident_str!("main").into(),
        ident_str!("new_capability").into(),
        vec![data_tag.clone()],
        vec![trail_arg, cap_arg, role, issued_to_none, valid_from_none, valid_until_none, clock],
    );
    let changes = execute_ptb(&client, &signer, ptb).await?;
    let logger_cap_id = find_created(&changes, "Capability")
        .into_iter()
        .find(|id| *id != admin_cap_id)
        .context("no new capability created")?;
    println!("  LoggerCap: {logger_cap_id}");

    // =========================================================
    // Phase 5: Create the AccessControllerBridge
    // =========================================================
    println!("\nPhase 5: Creating ACB...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, false).await?)?;
    let target_id = ptb.pure(trail_id)?;

    // Build capability_type_configs: VecMap<String, CapabilityTypeConfig>
    let type_name = ptb.pure("catch_logger".to_string())?;
    let prop_name = ptb_property_name(&mut ptb, hier_pkg, "catch_logging")?;
    let config = ptb_new_cap_type_config(&mut ptb, hier_pkg, acb_pkg_id, vec![prop_name]);

    let config_tag = type_tag(acb_pkg_id, "bridge", "CapabilityTypeConfig");
    let configs_map = ptb_vec_map_from_keys_values(
        &mut ptb, hier_pkg,
        string_type_tag(), config_tag,
        vec![type_name], vec![config],
    );

    // Phantom type: we need to pass a type argument but it must have `drop`.
    // Use 0x1::option::Option<bool> as a simple marker — any type with `drop` works.
    let marker_tag = iota_interaction::types::TypeTag::from_str("bool")?;

    let acb_result = ptb.programmable_move_call(
        acb_pkg_id,
        ident_str!("bridge").into(),
        ident_str!("create").into(),
        vec![marker_tag.clone()],
        vec![fed_arg, target_id, configs_map],
    );

    // Share the ACB using transfer::public_share_object
    let iota_framework = iota_interaction::types::base_types::ObjectID::from_hex_literal("0x2").unwrap();
    let acb_type_tag = iota_interaction::types::TypeTag::from_str(
        &format!("{acb_pkg_id}::bridge::AccessControllerBridge<bool>")
    )?;
    ptb.programmable_move_call(
        iota_framework,
        Identifier::new("transfer").unwrap(),
        Identifier::new("public_share_object").unwrap(),
        vec![acb_type_tag],
        vec![acb_result],
    );

    let changes = execute_ptb(&client, &signer, ptb).await?;
    let acb_id = find_created(&changes, "AccessControllerBridge")[0];
    println!("  ACB: {acb_id}");

    // =========================================================
    // Phase 6: Deposit capability into ACB
    // =========================================================
    println!("\nPhase 6: Depositing capability...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let acb_arg = ptb.obj(shared_obj_arg(&client, acb_id, true).await?)?;
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, false).await?)?;
    let cap_type = ptb.pure("catch_logger".to_string())?;
    let cap_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, logger_cap_id).await?))?;

    ptb.programmable_move_call(
        acb_pkg_id,
        ident_str!("bridge").into(),
        ident_str!("deposit_capability").into(),
        vec![marker_tag.clone()],
        vec![acb_arg, fed_arg, cap_type, cap_arg],
    );
    execute_ptb(&client, &signer, ptb).await?;
    println!("  Capability deposited");

    // =========================================================
    // Phase 7: Accredit attester (self-accredit for demo)
    // =========================================================
    println!("\nPhase 7: Accrediting attester...");

    let mut ptb = ProgrammableTransactionBuilder::new();
    let fed_arg = ptb.obj(shared_obj_arg(&client, federation_id, true).await?)?;
    let accredit_arg = ptb.obj(ObjectArg::ImmOrOwnedObject(owned_obj_ref(&client, accredit_cap_id).await?))?;
    let receiver_id = ptb.pure(sender)?;

    // Build properties vector for accreditation
    let prop_name = ptb_property_name(&mut ptb, hier_pkg, "catch_logging")?;
    let val_cod = ptb_property_value_string(&mut ptb, hier_pkg, "Cod")?;
    let val_haddock = ptb_property_value_string(&mut ptb, hier_pkg, "Haddock")?;

    let value_tag = type_tag(hier_pkg, "property_value", "PropertyValue");
    let vals_vec = ptb.command(Command::MakeMoveVec(Some(value_tag.clone().into()), vec![val_cod, val_haddock]));
    let allowed_set = ptb.programmable_move_call(
        hier_pkg,
        ident_str!("utils").into(),
        ident_str!("create_vec_set").into(),
        vec![value_tag],
        vec![vals_vec],
    );
    let allow_any = ptb.pure(false)?;
    let shape_tag = type_tag(hier_pkg, "property_shape", "PropertyShape");
    let shape_none = option_none(&mut ptb, shape_tag)?;

    let fed_property = ptb.programmable_move_call(
        hier_pkg,
        ident_str!("property").into(),
        ident_str!("new_property").into(),
        vec![],
        vec![prop_name, allowed_set, allow_any, shape_none],
    );

    let prop_tag = type_tag(hier_pkg, "property", "FederationProperty");
    let props_vec = ptb.command(Command::MakeMoveVec(Some(prop_tag.into()), vec![fed_property]));

    let clock = clock_arg(&mut ptb);
    ptb.programmable_move_call(
        hier_pkg,
        ident_str!("main").into(),
        ident_str!("create_accreditation_to_attest").into(),
        vec![],
        vec![fed_arg, accredit_arg, receiver_id, props_vec, clock],
    );
    execute_ptb(&client, &signer, ptb).await?;
    println!("  Attester accredited for catch_logging = [Cod, Haddock]");

    // =========================================================
    // Summary
    // =========================================================
    println!("\n=== Setup Complete ===");
    println!("Federation:         {federation_id}");
    println!("AuditTrail:         {trail_id}");
    println!("ACB:                {acb_id}");
    println!("AdminCap:           {admin_cap_id}");
    println!("RootAuthorityCap:   {root_cap_id}");
    println!("AccreditCap:        {accredit_cap_id}");
    println!("\nUse these IDs with 02_borrow_use_return example.");

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

fn option_none_string(ptb: &mut ProgrammableTransactionBuilder) -> anyhow::Result<Argument> {
    // Option<String> as None — can be passed as pure bcs
    Ok(ptb.pure(Option::<String>::None)?)
}
