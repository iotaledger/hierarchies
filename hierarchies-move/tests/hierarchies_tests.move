#[test_only]
module hierarchies::main_tests;

use hierarchies::{
    main::{
        new_federation,
        RootAuthorityCap,
        Federation,
        AccreditCap,
        add_property,
        revoke_accreditation_to_attest,
        revoke_accreditation_to_accredit,
        create_accreditation_to_accredit,
        create_accreditation_to_attest,
        add_root_authority,
        revoke_root_authority,
        is_root_authority,
        revoke_property
    },
    property,
    property_name::new_property_name,
    property_value::new_property_value_number
};
use iota::{clock, test_scenario, vec_map, vec_set};
use std::string::utf8;

#[test]
fun creating_new_federation_works() {
    let alice = @0x1;

    let mut scenario = test_scenario::begin(alice);

    // create new federation
    new_federation(scenario.ctx());

    scenario.next_tx(alice);

    // Check that the alice has RootAuthorityCap
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // check the federation
    let fed: Federation = scenario.take_shared();

    assert!(fed.is_accreditor(&alice.to_id()), 0);
    assert!(fed.is_attester(&alice.to_id()), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);

    let _ = scenario.end();
}

#[test]
fun test_adding_root_authority_to_the_federation() {
    let alice = @0x1;

    let mut scenario = test_scenario::begin(alice);

    let new_object = scenario.new_object();
    let bob = new_object.uid_to_inner();

    scenario.next_tx(alice);

    // Create a new federation
    new_federation(scenario.ctx());

    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a new root authority
    fed.add_root_authority(&cap, bob, scenario.ctx());

    scenario.next_tx(alice);

    // check that bob has RootAuthorityCap
    let bob_cap: RootAuthorityCap = scenario.take_from_address(bob.to_address());

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(bob.to_address(), bob_cap);
    test_scenario::return_shared(fed);
    new_object.delete();

    let _ = scenario.end();
}

#[test]
fun test_adding_trusted_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a Property
    let property_name = new_property_name(utf8(b"property_name"));
    let property_value = new_property_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.add_property(&cap, property, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the property was added
    assert!(fed.is_property_in_federation(property_name), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);

    let _ = scenario.end();
}

#[test]
fun test_create_accreditation() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let properties = vector::empty();
    fed.create_accreditation_to_accredit(
        &accredit_cap,
        bob,
        properties,
        &clock,
        scenario.ctx(),
    );
    scenario.next_tx(alice);

    // Check if the permission was issued
    assert!(fed.is_accreditor(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    clock.destroy_for_testing();
    new_id.delete();

    let _ = scenario.end();
}

#[test]
fun test_create_attestation() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);
    // Add a Property

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue accreditation to attest
    let properties = vector::empty();
    fed.create_accreditation_to_attest(&accredit_cap, bob, properties, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the accreditation was issued
    assert!(fed.is_attester(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    clock.destroy_for_testing();
    new_id.delete();

    let _ = scenario.end();
}

#[test]
fun test_revoke_accreditation_to_attest_and_accredit() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let properties = vector::empty();
    fed.create_accreditation_to_accredit(&accredit_cap, bob, properties, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Issue permission to attest
    fed.create_accreditation_to_attest(&accredit_cap, bob, properties, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to attest
    let permission_id = fed
        .get_accreditations_to_attest(&bob)
        .accredited_properties()[0]
        .id()
        .uid_to_inner();
    fed.revoke_accreditation_to_attest(&accredit_cap, &bob, &permission_id, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to accredit
    let permission_id = fed
        .get_accreditations_to_accredit(&bob)
        .accredited_properties()[0]
        .id()
        .uid_to_inner();
    fed.revoke_accreditation_to_accredit(
        &accredit_cap,
        &bob,
        &permission_id,
        &clock,
        scenario.ctx(),
    );
    scenario.next_tx(alice);

    // Check if the permission was revoked
    assert!(fed.is_attester(&bob), 0);
    assert!(fed.is_accreditor(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    clock.destroy_for_testing();
    new_id.delete();

    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EPropertyNotInFederation)]
fun test_create_accreditation_to_accredit_fails_for_nonexistent_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Create a property for a property that doesn't exist in the federation
    let nonexistent_property_name = new_property_name(utf8(b"nonexistent_role"));
    let allowed_values = vec_set::empty();
    let nonexistent_property = property::new_property(
        nonexistent_property_name,
        allowed_values,
        true,
        option::none(),
    );

    let properties = vector[nonexistent_property];

    // This should fail because the property name doesn't exist in the federation
    fed.create_accreditation_to_accredit(&accredit_cap, bob, properties, &clock, scenario.ctx());

    // Cleanup - this won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    new_id.delete();
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EPropertyNotInFederation)]
fun test_create_accreditation_to_attest_fails_for_nonexistent_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Create a property for a property that doesn't exist in the federation
    let nonexistent_property_name = new_property_name(utf8(b"nonexistent_role"));
    let allowed_values = vec_set::empty();
    let nonexistent_property = property::new_property(
        nonexistent_property_name,
        allowed_values,
        true,
        option::none(),
    );

    let properties = vector[nonexistent_property];

    // This should fail because the property name doesn't exist in the federation
    fed.create_accreditation_to_attest(&accredit_cap, bob, properties, &clock, scenario.ctx());

    // Cleanup - this won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    clock.destroy_for_testing();
    new_id.delete();
    let _ = scenario.end();
}

#[test]
fun test_create_accreditation_to_accredit_succeeds_for_existing_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // First add a property to the federation
    let property_name = new_property_name(utf8(b"role"));
    let property_value = new_property_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.add_property(&cap, property, scenario.ctx());
    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Create a property that matches the one we added to the federation
    let property_for_accreditation = property::new_property(
        property_name,
        allowed_values,
        false,
        option::none(),
    );

    let properties = vector[property_for_accreditation];

    // This should succeed because the property name exists in the federation
    fed.create_accreditation_to_accredit(&accredit_cap, bob, properties, &clock, scenario.ctx());

    scenario.next_tx(alice);

    // Verify the accreditation was created
    assert!(fed.is_accreditor(&bob), 0);

    // Cleanup
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    new_id.delete();
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
fun test_create_accreditation_to_attest_succeeds_for_existing_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // First add a property to the federation
    let property_name = new_property_name(utf8(b"role"));
    let property_value = new_property_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.add_property(&cap, property, scenario.ctx());
    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Create a property that matches the one we added to the federation
    let property_for_accreditation = property::new_property(
        property_name,
        allowed_values,
        false,
        option::none(),
    );

    let properties = vector[property_for_accreditation];

    // This should succeed because the property name exists in the federation
    fed.create_accreditation_to_attest(&accredit_cap, bob, properties, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Verify the accreditation was created
    assert!(fed.is_attester(&bob), 0);

    // Cleanup
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    new_id.delete();
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
fun test_revoke_root_authority_success() {
    let alice = @0x1;
    let bob = @0x2;
    let charlie = @0x3;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add Bob as root authority
    fed.add_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Add Charlie as root authority
    fed.add_root_authority(&alice_cap, charlie.to_id(), scenario.ctx());

    // Verify all three are root authorities
    assert!(fed.is_root_authority(&alice.to_id()), 0);
    assert!(fed.is_root_authority(&bob.to_id()), 1);
    assert!(fed.is_root_authority(&charlie.to_id()), 2);

    scenario.next_tx(alice);

    // Revoke Bob as root authority
    fed.revoke_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Verify Bob is no longer a root authority
    assert!(fed.is_root_authority(&alice.to_id()), 3);
    assert!(!fed.is_root_authority(&bob.to_id()), 4);
    assert!(fed.is_root_authority(&charlie.to_id()), 5);

    // Cleanup
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::ERootAuthorityNotFound)]
fun test_revoke_root_authority_not_found() {
    let alice = @0x1;
    let bob = @0x2;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Try to revoke Bob who is not a root authority
    fed.revoke_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::ECannotRevokeLastRootAuthority)]
fun test_cannot_revoke_last_root_authority() {
    let alice = @0x1;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Try to revoke the only root authority (Alice)
    fed.revoke_root_authority(&alice_cap, alice.to_id(), scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::ERevokedRootAuthority)]
fun test_revoked_authority_cannot_add_property() {
    let alice = @0x1;
    let bob = @0x2;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add Bob as root authority
    fed.add_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);

    // Bob gets his cap
    let bob_cap: RootAuthorityCap = scenario.take_from_address(bob);

    scenario.next_tx(alice);

    // Alice revokes Bob
    fed.revoke_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);

    // Bob tries to add a property with his revoked cap - should fail
    let property_name = new_property_name(utf8(b"test_property"));
    let allowed_values = vec_set::empty();
    let property = property::new_property(property_name, allowed_values, true, option::none());
    fed.add_property(&bob_cap, property, scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_to_address(bob, bob_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::ERevokedRootAuthority)]
fun test_revoked_authority_cannot_add_another_root_authority() {
    let alice = @0x1;
    let bob = @0x2;
    let charlie = @0x3;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add Bob as root authority
    fed.add_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);

    // Bob gets his cap
    let bob_cap: RootAuthorityCap = scenario.take_from_address(bob);

    scenario.next_tx(alice);

    // Alice revokes Bob
    fed.revoke_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);

    // Bob tries to add Charlie as root authority with his revoked cap - should fail
    fed.add_root_authority(&bob_cap, charlie.to_id(), scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_to_address(bob, bob_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_is_root_authority() {
    let alice = @0x1;
    let bob = @0x2;
    let charlie = @0x3;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Initially only Alice is a root authority
    assert!(fed.is_root_authority(&alice.to_id()), 0);
    assert!(!fed.is_root_authority(&bob.to_id()), 1);
    assert!(!fed.is_root_authority(&charlie.to_id()), 2);

    // Add Bob as root authority
    fed.add_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Now both Alice and Bob are root authorities
    assert!(fed.is_root_authority(&alice.to_id()), 3);
    assert!(fed.is_root_authority(&bob.to_id()), 4);
    assert!(!fed.is_root_authority(&charlie.to_id()), 5);

    // Cleanup
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EEmptyAllowedValuesWithoutAllowAny)]
fun test_add_property_with_empty_allowed_values_and_allow_any_false() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Try to add a property with empty allowed values and allow_any = false
    let property_name = new_property_name(utf8(b"invalid_property"));
    let allowed_values = vec_set::empty();

    // This should fail with EEmptyAllowedValuesWithoutAllowAny
    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.add_property(&cap, property, scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_add_property_with_empty_allowed_values_and_allow_any_true() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a property with empty allowed values and allow_any = true (should succeed)
    let property_name = new_property_name(utf8(b"any_value_property"));
    let allowed_values = vec_set::empty();

    let property = property::new_property(property_name, allowed_values, true, option::none());
    fed.add_property(&cap, property, scenario.ctx());

    // Verify the property was added
    assert!(fed.is_property_in_federation(property_name), 0);

    // Cleanup
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_add_property_with_allowed_values_and_allow_any_false() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a property with specific allowed values and allow_any = false (should succeed)
    let property_name = new_property_name(utf8(b"restricted_property"));
    let mut allowed_values = vec_set::empty();
    vec_set::insert(&mut allowed_values, new_property_value_number(1));
    vec_set::insert(&mut allowed_values, new_property_value_number(2));

    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.add_property(&cap, property, scenario.ctx());

    // Verify the property was added
    assert!(fed.is_property_in_federation(property_name), 0);

    // Cleanup
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[
    expected_failure(
        abort_code = hierarchies::main::EUnauthorizedInsufficientAccreditationToAccredit,
    ),
]
fun test_attester_cannot_revoke_attestation_rights() {
    let alice = @0x1;
    let bob = @0x2;
    let charlie = @0x3;

    let mut scenario = test_scenario::begin(alice);

    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let alice_accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // First add two different properties to the federation
    let property_name_1 = new_property_name(utf8(b"role1"));
    let property_name_2 = new_property_name(utf8(b"role2"));
    let property_value = new_property_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    let property_1 = property::new_property(property_name_1, allowed_values, false, option::none());
    let property_2 = property::new_property(property_name_2, allowed_values, false, option::none());
    fed.add_property(&root_cap, property_1, scenario.ctx());
    fed.add_property(&root_cap, property_2, scenario.ctx());
    scenario.next_tx(alice);

    // Create properties for permissions
    let property_1 = property::new_property(
        property_name_1,
        allowed_values,
        false,
        option::none(),
    );
    let property_2 = property::new_property(
        property_name_2,
        allowed_values,
        false,
        option::none(),
    );

    // Alice grants Charlie attestation rights for property_2
    let charlie_properties = vector[property_2];

    fed.create_accreditation_to_attest(
        &alice_accredit_cap,
        charlie.to_id(),
        charlie_properties,
        &clock,
        scenario.ctx(),
    );
    scenario.next_tx(alice);

    // Alice grants Bob accreditation rights only for property_1 (not property_2)
    let bob_properties = vector[property_1];
    fed.create_accreditation_to_accredit(
        &alice_accredit_cap,
        bob.to_id(),
        bob_properties,
        &clock,
        scenario.ctx(),
    );
    scenario.next_tx(bob);

    // Bob receives his AccreditCap
    let bob_accredit_cap: AccreditCap = scenario.take_from_address(bob);

    // Get Charlie's attestation permission ID (for property_2)
    let charlie_permission_id = fed
        .get_accreditations_to_attest(&charlie.to_id())
        .accredited_properties()[0]
        .id()
        .uid_to_inner();

    // Bob (who only has accreditation rights for property_1, not property_2)
    // tries to revoke Charlie's attestation rights for property_2 - this should fail
    fed.revoke_accreditation_to_attest(
        &bob_accredit_cap,
        &charlie.to_id(),
        &charlie_permission_id,
        &clock,
        scenario.ctx(),
    );

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, root_cap);
    test_scenario::return_to_address(alice, alice_accredit_cap);
    test_scenario::return_to_address(bob, bob_accredit_cap);
    test_scenario::return_shared(fed);
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EAlreadyRootAuthority)]
fun test_add_already_existing_root_authority() {
    let alice = @0x1;
    let bob = @0x2;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add Bob as root authority
    fed.add_root_authority(&cap, bob.to_id(), scenario.ctx());

    // Try to add Bob again - should fail
    fed.add_root_authority(&cap, bob.to_id(), scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_reinstate_root_authority_success() {
    let alice = @0x1;
    let bob = @0x2;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add Bob as root authority
    fed.add_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);
    let bob_cap: RootAuthorityCap = scenario.take_from_address(bob);

    scenario.next_tx(alice);

    // Alice revokes Bob
    fed.revoke_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Verify Bob is no longer a root authority
    assert!(!fed.is_root_authority(&bob.to_id()), 0);

    // Alice reinstates Bob
    fed.reinstate_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Verify Bob is now a root authority again
    assert!(fed.is_root_authority(&bob.to_id()), 0);

    // Cleanup
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_to_address(bob, bob_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::ENotRevokedRootAuthority)]
fun test_reinstate_non_revoked_authority() {
    let alice = @0x1;
    let bob = @0x2;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Try to reinstate Bob who was never revoked - should fail
    fed.reinstate_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EAlreadyRootAuthority)]
fun test_reinstate_already_active_authority() {
    let alice = @0x1;
    let bob = @0x2;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add Bob as root authority
    fed.add_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);
    let bob_cap: RootAuthorityCap = scenario.take_from_address(bob);

    scenario.next_tx(alice);

    // Try to reinstate Bob who is already active - should fail
    fed.reinstate_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_to_address(bob, bob_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_reinstated_authority_can_perform_actions() {
    let alice = @0x1;
    let bob = @0x2;

    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add Bob as root authority
    fed.add_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);
    let bob_cap: RootAuthorityCap = scenario.take_from_address(bob);

    scenario.next_tx(alice);

    // Alice revokes Bob
    fed.revoke_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    // Alice reinstates Bob
    fed.reinstate_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);

    // Bob should be able to add a property with his reinstated authority
    let property_name = new_property_name(utf8(b"test_property"));
    let allowed_values = vec_set::empty();
    let property = property::new_property(property_name, allowed_values, true, option::none());
    fed.add_property(&bob_cap, property, scenario.ctx());

    // Verify the property was added
    assert!(fed.is_property_in_federation(property_name), 0);

    // Cleanup
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_to_address(bob, bob_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EPropertyRevoked)]
fun test_create_accreditation_to_accredit_fails_for_revoked_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    let property_name = new_property_name(utf8(b"role"));
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(new_property_value_number(1));
    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.add_property(&root_cap, property, scenario.ctx());

    fed.revoke_property(&root_cap, property_name, &clock, scenario.ctx());

    let stmt = property::new_property(property_name, vec_set::empty(), true, option::none());
    fed.create_accreditation_to_accredit(
        &accredit_cap,
        @0x2.to_id(),
        vector[stmt],
        &clock,
        scenario.ctx(),
    );

    test_scenario::return_shared(fed);
    test_scenario::return_to_address(alice, root_cap);
    test_scenario::return_to_address(alice, accredit_cap);
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EPropertyRevoked)]
fun test_create_accreditation_to_attest_fails_for_revoked_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    let property_name = new_property_name(utf8(b"role"));
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(new_property_value_number(1));
    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.add_property(&root_cap, property, scenario.ctx());

    fed.revoke_property(&root_cap, property_name, &clock, scenario.ctx());

    let property = property::new_property(property_name, vec_set::empty(), true, option::none());
    fed.create_accreditation_to_attest(
        &accredit_cap,
        @0x2.to_id(),
        vector[property],
        &clock,
        scenario.ctx(),
    );

    test_scenario::return_shared(fed);
    test_scenario::return_to_address(alice, root_cap);
    test_scenario::return_to_address(alice, accredit_cap);
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::ERevokedRootAuthority)]
fun test_transferred_capability_from_revoked_authority_fails() {
    let alice = @0x1;
    let bob = @0x2;
    let charlie = @0x3;

    let mut scenario = test_scenario::begin(alice);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let alice_cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add Bob as root authority
    fed.add_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);
    let bob_cap: RootAuthorityCap = scenario.take_from_address(bob);

    scenario.next_tx(alice);

    // Alice revokes Bob
    fed.revoke_root_authority(&alice_cap, bob.to_id(), scenario.ctx());

    scenario.next_tx(bob);

    // Bob transfers his capability to Charlie
    fed.transfer_root_authority_cap(bob_cap, charlie.to_id(), scenario.ctx());

    scenario.next_tx(charlie);
    let transferred_cap: RootAuthorityCap = scenario.take_from_address(charlie);

    // Charlie tries to use Bob's (revoked) capability - should fail
    let property_name = new_property_name(utf8(b"test_property"));
    let allowed_values = vec_set::empty();
    let property = property::new_property(property_name, allowed_values, true, option::none());
    fed.add_property(&transferred_cap, property, scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_to_address(charlie, transferred_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_validate_property_fails_for_revoked_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // Add a property to the federation
    let property_name = new_property_name(utf8(b"role"));
    let property_value = new_property_value_number(1);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);
    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.add_property(&root_cap, property, scenario.ctx());

    // Create accreditation for Bob to attest this property
    let bob_id = @0x2.to_id();
    let property = property::new_property(property_name, allowed_values, false, option::none());
    fed.create_accreditation_to_attest(
        &accredit_cap,
        bob_id,
        vector[property],
        &clock,
        scenario.ctx(),
    );

    // Initially validation should pass
    assert!(fed.validate_property(&bob_id, property_name, property_value, &clock), 0);

    // Revoke the property
    fed.revoke_property(&root_cap, property_name, &clock, scenario.ctx());

    // Now validation should fail because the property is revoked
    assert!(!fed.validate_property(&bob_id, property_name, property_value, &clock), 1);

    test_scenario::return_shared(fed);
    test_scenario::return_to_address(alice, root_cap);
    test_scenario::return_to_address(alice, accredit_cap);
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
fun test_validate_properties_fails_for_revoked_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // Add two properties to the federation
    let property_name_1 = new_property_name(utf8(b"role1"));
    let property_name_2 = new_property_name(utf8(b"role2"));
    let property_value_1 = new_property_value_number(1);
    let property_value_2 = new_property_value_number(2);
    let mut allowed_values_1 = vec_set::empty();
    let mut allowed_values_2 = vec_set::empty();
    allowed_values_1.insert(property_value_1);
    allowed_values_2.insert(property_value_2);

    let property_1 = property::new_property(
        property_name_1,
        allowed_values_1,
        false,
        option::none(),
    );
    let property_2 = property::new_property(
        property_name_2,
        allowed_values_2,
        false,
        option::none(),
    );
    fed.add_property(&root_cap, property_1, scenario.ctx());
    fed.add_property(&root_cap, property_2, scenario.ctx());

    // Create accreditation for Bob to attest both properties
    let bob_id = @0x2.to_id();
    let property_1 = property::new_property(
        property_name_1,
        allowed_values_1,
        false,
        option::none(),
    );
    let property_2 = property::new_property(
        property_name_2,
        allowed_values_2,
        false,
        option::none(),
    );
    fed.create_accreditation_to_attest(
        &accredit_cap,
        bob_id,
        vector[property_1, property_2],
        &clock,
        scenario.ctx(),
    );

    // Create property map for validation
    let mut properties = vec_map::empty();
    properties.insert(property_name_1, property_value_1);
    properties.insert(property_name_2, property_value_2);

    // Initially validation should pass
    assert!(fed.validate_properties(&bob_id, properties, &clock), 0);

    // Revoke the first property
    fed.revoke_property(&root_cap, property_name_1, &clock, scenario.ctx());

    // Now validation should fail because one of the properties is revoked
    assert!(!fed.validate_properties(&bob_id, properties, &clock), 1);

    test_scenario::return_shared(fed);
    test_scenario::return_to_address(alice, root_cap);
    test_scenario::return_to_address(alice, accredit_cap);
    clock.destroy_for_testing();
    let _ = scenario.end();
}
