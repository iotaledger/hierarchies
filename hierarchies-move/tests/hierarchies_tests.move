#[test_only]
module hierarchies::main_tests;

use hierarchies::{
    main::{
        new_federation,
        RootAuthorityCap,
        Federation,
        AccreditCap,
        AttestCap,
        add_statement,
        revoke_accreditation_to_attest,
        revoke_accreditation_to_accredit,
        create_accreditation_to_accredit,
        create_accreditation_to_attest,
        add_root_authority,
        revoke_root_authority,
        is_root_authority
    },
    statement,
    statement_name::new_statement_name,
    statement_value::new_statement_value_number
};
use iota::{test_scenario, vec_set};
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
fun test_adding_trusted_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a Statement
    let statement_name = new_statement_name(utf8(b"statement_name"));
    let property_value = new_statement_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    fed.add_statement(&cap, statement_name, allowed_values, false, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the property was added
    assert!(fed.is_statement_in_federation(statement_name), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);

    let _ = scenario.end();
}

#[test]
fun test_create_accreditation() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

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
    let statements = vector::empty();
    fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was issued
    assert!(fed.is_accreditor(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
}

#[test]
fun test_create_attestation() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let attest_cap: AttestCap = scenario.take_from_address(alice);
    // Add a Statement

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let statements = vector::empty();
    fed.create_accreditation_to_attest(&attest_cap, bob, statements, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was issued
    assert!(fed.is_attester(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
}

#[test]
fun test_revoke_accreditation_to_attest_and_accredit() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);
    let attest_cap: AttestCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let statements = vector::empty();
    fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, scenario.ctx());
    scenario.next_tx(alice);

    // Issue permission to attest
    fed.create_accreditation_to_attest(&attest_cap, bob, statements, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to attest
    let permission_id = fed
        .get_accreditations_to_attest(&bob)
        .accredited_statements()[0]
        .id()
        .uid_to_inner();
    fed.revoke_accreditation_to_attest(&attest_cap, &bob, &permission_id, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to accredit
    let permission_id = fed
        .get_accreditations_to_accredit(&bob)
        .accredited_statements()[0]
        .id()
        .uid_to_inner();
    fed.revoke_accreditation_to_accredit(&accredit_cap, &bob, &permission_id, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was revoked
    // TODO::@itsyaasir: This should be fixed since the user has no permissions
    // and should not be able to attest/accredit
    assert!(fed.is_attester(&bob), 0);
    assert!(fed.is_accreditor(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EStatementNotInFederation)]
fun test_create_accreditation_to_accredit_fails_for_nonexistent_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Create a statement for a property that doesn't exist in the federation
    let nonexistent_statement_name = new_statement_name(utf8(b"nonexistent_role"));
    let allowed_values = vec_set::empty();
    let nonexistent_statement = statement::new_statement(
        nonexistent_statement_name,
        allowed_values,
        true,
        option::none(),
    );

    let statements = vector[nonexistent_statement];

    // This should fail because the statement name doesn't exist in the federation
    fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, scenario.ctx());

    // Cleanup - this won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    new_id.delete();
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EStatementNotInFederation)]
fun test_create_accreditation_to_attest_fails_for_nonexistent_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let attest_cap: AttestCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Create a statement for a property that doesn't exist in the federation
    let nonexistent_statement_name = new_statement_name(utf8(b"nonexistent_role"));
    let allowed_values = vec_set::empty();
    let nonexistent_statement = statement::new_statement(
        nonexistent_statement_name,
        allowed_values,
        true,
        option::none(),
    );

    let statements = vector[nonexistent_statement];

    // This should fail because the statement name doesn't exist in the federation
    fed.create_accreditation_to_attest(&attest_cap, bob, statements, scenario.ctx());

    // Cleanup - this won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_shared(fed);
    new_id.delete();
    let _ = scenario.end();
}

#[test]
fun test_create_accreditation_to_accredit_succeeds_for_existing_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // First add a statement to the federation
    let statement_name = new_statement_name(utf8(b"role"));
    let property_value = new_statement_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    fed.add_statement(&cap, statement_name, allowed_values, false, scenario.ctx());
    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Create a statement that matches the one we added to the federation
    let statement_for_accreditation = statement::new_statement(
        statement_name,
        allowed_values,
        false,
        option::none(),
    );

    let statements = vector[statement_for_accreditation];

    // This should succeed because the statement name exists in the federation
    fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, scenario.ctx());
    scenario.next_tx(alice);

    // Verify the accreditation was created
    assert!(fed.is_accreditor(&bob), 0);

    // Cleanup
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    new_id.delete();
    let _ = scenario.end();
}

#[test]
fun test_create_accreditation_to_attest_succeeds_for_existing_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let attest_cap: AttestCap = scenario.take_from_address(alice);

    // First add a statement to the federation
    let statement_name = new_statement_name(utf8(b"role"));
    let property_value = new_statement_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    fed.add_statement(&cap, statement_name, allowed_values, false, scenario.ctx());
    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Create a statement that matches the one we added to the federation
    let statement_for_accreditation = statement::new_statement(
        statement_name,
        allowed_values,
        false,
        option::none(),
    );

    let statements = vector[statement_for_accreditation];

    // This should succeed because the statement name exists in the federation
    fed.create_accreditation_to_attest(&attest_cap, bob, statements, scenario.ctx());
    scenario.next_tx(alice);

    // Verify the accreditation was created
    assert!(fed.is_attester(&bob), 0);

    // Cleanup
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_shared(fed);
    new_id.delete();
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
fun test_revoked_authority_cannot_add_statement() {
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

    // Bob tries to add a statement with his revoked cap - should fail
    let statement_name = new_statement_name(utf8(b"test_statement"));
    let allowed_values = vec_set::empty();
    fed.add_statement(&bob_cap, statement_name, allowed_values, true, scenario.ctx());

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
fun test_add_statement_with_empty_allowed_values_and_allow_any_false() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Try to add a statement with empty allowed values and allow_any = false
    let statement_name = new_statement_name(utf8(b"invalid_statement"));
    let allowed_values = vec_set::empty();

    // This should fail with EEmptyAllowedValuesWithoutAllowAny
    fed.add_statement(&cap, statement_name, allowed_values, false, scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_add_statement_with_empty_allowed_values_and_allow_any_true() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a statement with empty allowed values and allow_any = true (should succeed)
    let statement_name = new_statement_name(utf8(b"any_value_statement"));
    let allowed_values = vec_set::empty();

    fed.add_statement(&cap, statement_name, allowed_values, true, scenario.ctx());

    // Verify the statement was added
    assert!(fed.is_statement_in_federation(statement_name), 0);

    // Cleanup
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_add_statement_with_allowed_values_and_allow_any_false() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a statement with specific allowed values and allow_any = false (should succeed)
    let statement_name = new_statement_name(utf8(b"restricted_statement"));
    let mut allowed_values = vec_set::empty();
    vec_set::insert(&mut allowed_values, new_statement_value_number(1));
    vec_set::insert(&mut allowed_values, new_statement_value_number(2));

    fed.add_statement(&cap, statement_name, allowed_values, false, scenario.ctx());

    // Verify the statement was added
    assert!(fed.is_statement_in_federation(statement_name), 0);

    // Cleanup
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}
