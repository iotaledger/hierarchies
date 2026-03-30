#[test_only]
module access_controller_bridge::bridge_tests;

use access_controller_bridge::{
    bridge::{Self, AccessControllerBridge},
    test_utils::{
        Self,
        TestMarker,
        alice, bob,
        catch_logging_name, catch_management_name,
        cod_value,
    },
};
use hierarchies::main::Federation;
use hierarchies::property_value;
use tf_components::capability::{Self as capability};
use iota::{clock, test_scenario::{Self as ts}, vec_map};
use std::string::utf8;

// ===== Helpers =====

/// Full setup: federation + capabilities + ACB + deposits + accreditation.
/// The ACB targets a fake ID; the Capabilities are created via role_map.
fun full_setup(scenario: &mut ts::Scenario): ID {
    test_utils::setup_federation(scenario);

    // Create a target_key (simulating an audit trail object ID)
    let target_uid = scenario.new_object();
    let target_id = target_uid.uid_to_inner();
    target_uid.delete();

    // Create capabilities directly via role_map
    let (role_map, admin_cap, logger_cap, manager_cap) =
        test_utils::create_test_capabilities(target_id, scenario.ctx());

    // Transfer caps to ALICE for deposit
    transfer::public_transfer(logger_cap, alice());
    transfer::public_transfer(manager_cap, alice());
    capability::destroy_for_testing(admin_cap);
    test_utils::destroy_role_map(role_map);

    scenario.next_tx(alice());

    // Create ACB
    let fed: Federation = ts::take_shared(scenario);
    let mut configs = vec_map::empty();
    configs.insert(
        utf8(b"catch_logger"),
        bridge::new_capability_type_config(vector[catch_logging_name()]),
    );
    configs.insert(
        utf8(b"catch_manager"),
        bridge::new_capability_type_config(vector[
            catch_logging_name(),
            catch_management_name(),
        ]),
    );

    let acb = bridge::create<TestMarker>(&fed, target_id, configs, scenario.ctx());
    transfer::public_share_object(acb);
    ts::return_shared(fed);
    scenario.next_tx(alice());

    // Deposit logger cap
    let fed: Federation = ts::take_shared(scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(scenario);
    let cap1: capability::Capability = ts::take_from_address(scenario, alice());
    bridge::deposit_capability(&mut acb, &fed, utf8(b"catch_logger"), cap1, scenario.ctx());
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.next_tx(alice());

    // Deposit manager cap
    let fed: Federation = ts::take_shared(scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(scenario);
    let cap2: capability::Capability = ts::take_from_address(scenario, alice());
    bridge::deposit_capability(&mut acb, &fed, utf8(b"catch_manager"), cap2, scenario.ctx());
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.next_tx(alice());

    // Accredit BOB
    test_utils::accredit_bob_as_attester(scenario);

    target_id
}

// ===== Create / Delete Tests =====

#[test]
fun test_create_and_query() {
    let mut scenario = ts::begin(alice());
    test_utils::setup_federation(&mut scenario);

    let target_uid = scenario.new_object();
    let target_id = target_uid.uid_to_inner();
    target_uid.delete();

    let fed: Federation = ts::take_shared(&scenario);
    let mut configs = vec_map::empty();
    configs.insert(
        utf8(b"logger"),
        bridge::new_capability_type_config(vector[catch_logging_name()]),
    );

    let acb = bridge::create<TestMarker>(&fed, target_id, configs, scenario.ctx());

    assert!(bridge::target_id(&acb) == target_id);
    assert!(bridge::federation_id(&acb) == object::id(&fed));
    assert!(!bridge::is_frozen(&acb));
    assert!(bridge::version(&acb) == 1);
    assert!(bridge::has_capability_type(&acb, &utf8(b"logger")));
    assert!(!bridge::has_capability_type(&acb, &utf8(b"other")));

    transfer::public_share_object(acb);
    ts::return_shared(fed);
    scenario.next_tx(alice());

    // Delete it (no caps deposited)
    let fed: Federation = ts::take_shared(&scenario);
    let acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    bridge::delete(acb, &fed, scenario.ctx());
    ts::return_shared(fed);

    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::ENotRootAuthority)]
fun test_create_not_root_authority_fails() {
    let mut scenario = ts::begin(alice());
    test_utils::setup_federation(&mut scenario);

    scenario.next_tx(bob());
    let fed: Federation = ts::take_shared(&scenario);
    let mut configs = vec_map::empty();
    configs.insert(
        utf8(b"logger"),
        bridge::new_capability_type_config(vector[catch_logging_name()]),
    );

    let acb = bridge::create<TestMarker>(&fed, @0xDEAD.to_id(), configs, scenario.ctx());
    transfer::public_share_object(acb);
    ts::return_shared(fed);
    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::EEmptyRequiredProperties)]
fun test_empty_properties_fails() {
    let scenario = ts::begin(alice());
    // Creating config with empty vector should fail
    bridge::new_capability_type_config(vector[]);
    scenario.end();
}

// ===== Deposit / Withdraw Tests =====

#[test]
fun test_deposit_and_withdraw() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    // Verify deposited and withdraw in one take
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    assert!(bridge::is_capability_deposited(&acb, &utf8(b"catch_logger")));
    assert!(bridge::is_capability_deposited(&acb, &utf8(b"catch_manager")));

    let cap = bridge::withdraw_capability(&mut acb, &fed, utf8(b"catch_logger"), scenario.ctx());
    assert!(!bridge::is_capability_deposited(&acb, &utf8(b"catch_logger")));

    capability::destroy_for_testing(cap);
    ts::return_shared(acb);
    ts::return_shared(fed);

    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::ECapabilityTargetMismatch)]
fun test_deposit_wrong_target_fails() {
    let mut scenario = ts::begin(alice());
    test_utils::setup_federation(&mut scenario);

    // Create cap for one target
    let target_uid = scenario.new_object();
    let target_id = target_uid.uid_to_inner();
    target_uid.delete();
    let (role_map, admin_cap, logger_cap, manager_cap) =
        test_utils::create_test_capabilities(target_id, scenario.ctx());
    transfer::public_transfer(logger_cap, alice());
    capability::destroy_for_testing(admin_cap);
    capability::destroy_for_testing(manager_cap);
    test_utils::destroy_role_map(role_map);
    scenario.next_tx(alice());

    // Create ACB with a different target
    let fed: Federation = ts::take_shared(&scenario);
    let mut configs = vec_map::empty();
    configs.insert(
        utf8(b"catch_logger"),
        bridge::new_capability_type_config(vector[catch_logging_name()]),
    );
    let acb = bridge::create<TestMarker>(&fed, @0xDEAD.to_id(), configs, scenario.ctx());
    transfer::public_share_object(acb);
    ts::return_shared(fed);
    scenario.next_tx(alice());

    // Deposit cap with wrong target
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let cap: capability::Capability = ts::take_from_address(&scenario, alice());
    bridge::deposit_capability(&mut acb, &fed, utf8(b"catch_logger"), cap, scenario.ctx());
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

// ===== Borrow / Return Tests =====

#[test]
fun test_borrow_and_return() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    // Bob borrows as attester
    scenario.next_tx(bob());
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);

    let mut props = vec_map::empty();
    props.insert(catch_logging_name(), cod_value());

    let (cap, receipt) = bridge::borrow(
        &mut acb, &fed, utf8(b"catch_logger"), props, &clock, scenario.ctx(),
    );

    // Capability is removed from ACB while borrowed
    assert!(!bridge::is_capability_deposited(&acb, &utf8(b"catch_logger")));

    // Return cap
    bridge::return_cap(&mut acb, cap, receipt, &clock);

    // Capability is back in ACB
    assert!(bridge::is_capability_deposited(&acb, &utf8(b"catch_logger")));

    clock::destroy_for_testing(clock);
    ts::return_shared(acb);
    ts::return_shared(fed);

    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::EValidationFailed)]
fun test_borrow_non_attester_fails() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    let charlie = @0xC;
    scenario.next_tx(charlie);
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);

    let mut props = vec_map::empty();
    props.insert(catch_logging_name(), cod_value());

    let (cap, receipt) = bridge::borrow(
        &mut acb, &fed, utf8(b"catch_logger"), props, &clock, scenario.ctx(),
    );

    bridge::return_cap(&mut acb, cap, receipt, &clock);
    clock::destroy_for_testing(clock);
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::EValidationFailed)]
fun test_borrow_wrong_value_fails() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    scenario.next_tx(bob());
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);

    let mut props = vec_map::empty();
    props.insert(catch_logging_name(), property_value::new_property_value_string(utf8(b"Mackerel")));

    let (cap, receipt) = bridge::borrow(
        &mut acb, &fed, utf8(b"catch_logger"), props, &clock, scenario.ctx(),
    );

    bridge::return_cap(&mut acb, cap, receipt, &clock);
    clock::destroy_for_testing(clock);
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::ECapabilityTypeNotFound)]
fun test_borrow_unknown_type_fails() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    scenario.next_tx(bob());
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);

    let (cap, receipt) = bridge::borrow(
        &mut acb, &fed, utf8(b"nonexistent"), vec_map::empty(), &clock, scenario.ctx(),
    );

    bridge::return_cap(&mut acb, cap, receipt, &clock);
    clock::destroy_for_testing(clock);
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::EPropertyNotProvided)]
fun test_borrow_missing_property_fails() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    scenario.next_tx(bob());
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);

    // catch_manager requires both properties, only provide one
    let mut props = vec_map::empty();
    props.insert(catch_logging_name(), cod_value());

    let (cap, receipt) = bridge::borrow(
        &mut acb, &fed, utf8(b"catch_manager"), props, &clock, scenario.ctx(),
    );

    bridge::return_cap(&mut acb, cap, receipt, &clock);
    clock::destroy_for_testing(clock);
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::EBridgeFrozen)]
fun test_borrow_frozen_fails() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    // Freeze
    scenario.next_tx(alice());
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);
    bridge::emergency_freeze(&mut acb, &fed, &clock, scenario.ctx());
    ts::return_shared(acb);
    ts::return_shared(fed);
    clock::destroy_for_testing(clock);

    // Bob tries to borrow — fails
    scenario.next_tx(bob());
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);

    let mut props = vec_map::empty();
    props.insert(catch_logging_name(), cod_value());

    let (cap, receipt) = bridge::borrow(
        &mut acb, &fed, utf8(b"catch_logger"), props, &clock, scenario.ctx(),
    );

    bridge::return_cap(&mut acb, cap, receipt, &clock);
    clock::destroy_for_testing(clock);
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

// ===== Lifecycle Tests =====

#[test]
fun test_freeze_and_unfreeze() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);

    bridge::emergency_freeze(&mut acb, &fed, &clock, scenario.ctx());
    assert!(bridge::is_frozen(&acb));

    bridge::emergency_unfreeze(&mut acb, &fed, &clock, scenario.ctx());
    assert!(!bridge::is_frozen(&acb));

    clock::destroy_for_testing(clock);
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

#[test]
fun test_add_and_remove_capability_type() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);

    bridge::add_capability_type(
        &mut acb, &fed,
        utf8(b"auditor"),
        bridge::new_capability_type_config(vector[catch_logging_name()]),
        scenario.ctx(),
    );
    assert!(bridge::has_capability_type(&acb, &utf8(b"auditor")));

    bridge::remove_capability_type(&mut acb, &fed, utf8(b"auditor"), scenario.ctx());
    assert!(!bridge::has_capability_type(&acb, &utf8(b"auditor")));

    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

#[test]
fun test_update_capability_type_config() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);

    bridge::update_capability_type_config(
        &mut acb, &fed,
        utf8(b"catch_logger"),
        bridge::new_capability_type_config(vector[
            catch_logging_name(),
            catch_management_name(),
        ]),
        scenario.ctx(),
    );

    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

#[test]
#[expected_failure(abort_code = bridge::ECapabilityAlreadyDeposited)]
fun test_remove_type_with_deposited_cap_fails() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    bridge::remove_capability_type(&mut acb, &fed, utf8(b"catch_logger"), scenario.ctx());
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}

// Note: Revocation tests (revoking accreditation → borrow fails) require access
// to `accreditation::accredited_properties()` which is `public(package)`.
// This scenario is validated in the localnet integration examples.

// ===== Double borrow test =====

#[test]
#[expected_failure(abort_code = bridge::ECapabilityCurrentlyBorrowed)]
fun test_double_borrow_fails() {
    let mut scenario = ts::begin(alice());
    full_setup(&mut scenario);

    // Bob borrows catch_logger
    scenario.next_tx(bob());
    let fed: Federation = ts::take_shared(&scenario);
    let mut acb: AccessControllerBridge<TestMarker> = ts::take_shared(&scenario);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(2000);

    let mut props = vec_map::empty();
    props.insert(catch_logging_name(), cod_value());

    let (cap1, receipt1) = bridge::borrow(
        &mut acb, &fed, utf8(b"catch_logger"), props, &clock, scenario.ctx(),
    );

    // Try to borrow again — should fail (already borrowed)
    let mut props2 = vec_map::empty();
    props2.insert(catch_logging_name(), cod_value());

    let (cap2, receipt2) = bridge::borrow(
        &mut acb, &fed, utf8(b"catch_logger"), props2, &clock, scenario.ctx(),
    );

    bridge::return_cap(&mut acb, cap2, receipt2, &clock);
    bridge::return_cap(&mut acb, cap1, receipt1, &clock);
    clock::destroy_for_testing(clock);
    ts::return_shared(acb);
    ts::return_shared(fed);
    scenario.end();
}
