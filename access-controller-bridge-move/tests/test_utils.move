#[test_only]
/// Shared test helpers for Access Controller Bridge tests.
///
/// Creates Capabilities directly via tf_components::role_map, avoiding
/// any dependency on the audit trail package.
module access_controller_bridge::test_utils;

use hierarchies::{
    main::{Self as hierarchies, Federation, RootAuthorityCap, AccreditCap},
    property,
    property_name::{Self, PropertyName},
    property_value::{Self, PropertyValue},
};
use tf_components::{
    capability::Capability,
    role_map::{Self, RoleMap},
};
use iota::{clock, test_scenario::{Self as ts, Scenario}, vec_set};
use std::string::utf8;

// ===== Phantom marker for tests =====

public struct TestMarker has drop {}

// ===== Test permission type for RoleMap =====

public enum TestPermission has copy, drop, store {
    Admin,
    Write,
    Read,
}

// ===== Constants =====

const ALICE: address = @0xA;
const BOB: address = @0xB;

public fun alice(): address { ALICE }
public fun bob(): address { BOB }

// ===== Property helpers =====

public fun catch_logging_name(): PropertyName {
    property_name::new_property_name(utf8(b"catch_logging"))
}

public fun catch_management_name(): PropertyName {
    property_name::new_property_name(utf8(b"catch_management"))
}

public fun cod_value(): PropertyValue {
    property_value::new_property_value_string(utf8(b"Cod"))
}

public fun haddock_value(): PropertyValue {
    property_value::new_property_value_string(utf8(b"Haddock"))
}

// ===== Setup: Federation with properties =====

/// Creates a federation with catch_logging and catch_management properties.
/// After this, ALICE owns RootAuthorityCap and AccreditCap.
public fun setup_federation(scenario: &mut Scenario) {
    hierarchies::new_federation(scenario.ctx());
    scenario.next_tx(ALICE);

    let mut fed: Federation = ts::take_shared(scenario);
    let root_cap: RootAuthorityCap = ts::take_from_address(scenario, ALICE);

    fed.add_property(
        &root_cap,
        property::new_property(
            catch_logging_name(),
            vec_set::from_keys(vector[
                property_value::new_property_value_string(utf8(b"Cod")),
                property_value::new_property_value_string(utf8(b"Haddock")),
            ]),
            false,
            option::none(),
        ),
        scenario.ctx(),
    );

    fed.add_property(
        &root_cap,
        property::new_property(
            catch_management_name(),
            vec_set::empty(),
            true,
            option::none(),
        ),
        scenario.ctx(),
    );

    ts::return_shared(fed);
    ts::return_to_address(ALICE, root_cap);
    scenario.next_tx(ALICE);
}

// ===== Setup: Create a standalone RoleMap + Capabilities =====

/// Creates a RoleMap with "catch_logger" and "catch_manager" roles,
/// mints bearer Capabilities for each, and returns everything.
///
/// The target_key is the object the capabilities authorize access to.
public fun create_test_capabilities(
    target_key: ID,
    ctx: &mut TxContext,
): (RoleMap<TestPermission, u8>, Capability, Capability, Capability) {
    let admin_permissions = vec_set::from_keys(vector[
        TestPermission::Admin,
        TestPermission::Write,
        TestPermission::Read,
    ]);

    let (mut role_map, admin_cap) = role_map::new<TestPermission, u8>(
        target_key,
        utf8(b"Admin"),
        admin_permissions,
        role_map::new_role_admin_permissions(
            TestPermission::Admin,
            TestPermission::Admin,
            TestPermission::Admin,
        ),
        role_map::new_capability_admin_permissions(
            TestPermission::Admin,
            TestPermission::Admin,
        ),
        ctx,
    );

    // Create catch_logger role
    let mut clock = clock::create_for_testing(ctx);
    clock.set_for_testing(1000);

    role_map.create_role(
        &admin_cap,
        utf8(b"catch_logger"),
        vec_set::from_keys(vector[TestPermission::Write]),
        option::none(),
        &clock,
        ctx,
    );

    role_map.create_role(
        &admin_cap,
        utf8(b"catch_manager"),
        vec_set::from_keys(vector[TestPermission::Write, TestPermission::Read]),
        option::none(),
        &clock,
        ctx,
    );

    // Mint bearer capabilities
    let logger_cap = role_map.new_capability(
        &admin_cap,
        &utf8(b"catch_logger"),
        option::none(),
        option::none(),
        option::none(),
        &clock,
        ctx,
    );

    let manager_cap = role_map.new_capability(
        &admin_cap,
        &utf8(b"catch_manager"),
        option::none(),
        option::none(),
        option::none(),
        &clock,
        ctx,
    );

    clock::destroy_for_testing(clock);

    (role_map, admin_cap, logger_cap, manager_cap)
}

/// Destroy a RoleMap used for testing.
public fun destroy_role_map(role_map: RoleMap<TestPermission, u8>) {
    role_map.destroy();
}

// ===== Accreditation =====

/// Accredit BOB as attester for catch_logging = [Cod, Haddock].
public fun accredit_bob_as_attester(scenario: &mut Scenario) {
    let mut fed: Federation = ts::take_shared(scenario);
    let accredit_cap: AccreditCap = ts::take_from_address(scenario, ALICE);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    hierarchies::create_accreditation_to_attest(
        &mut fed,
        &accredit_cap,
        BOB.to_id(),
        vector[property::new_property(
            catch_logging_name(),
            vec_set::from_keys(vector[cod_value(), haddock_value()]),
            false,
            option::none(),
        )],
        &clock,
        scenario.ctx(),
    );

    clock::destroy_for_testing(clock);
    ts::return_to_address(ALICE, accredit_cap);
    ts::return_shared(fed);
    scenario.next_tx(ALICE);
}

/// Accredit BOB for catch_management (allow_any) as well.
public fun accredit_bob_as_manager(scenario: &mut Scenario) {
    let mut fed: Federation = ts::take_shared(scenario);
    let accredit_cap: AccreditCap = ts::take_from_address(scenario, ALICE);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    hierarchies::create_accreditation_to_attest(
        &mut fed,
        &accredit_cap,
        BOB.to_id(),
        vector[property::new_property(
            catch_management_name(),
            vec_set::empty(),
            true,
            option::none(),
        )],
        &clock,
        scenario.ctx(),
    );

    clock::destroy_for_testing(clock);
    ts::return_to_address(ALICE, accredit_cap);
    ts::return_shared(fed);
    scenario.next_tx(ALICE);
}
