#[test_only]
module hierarchies::main_tests;

use hierarchies::{
    main::{
        new_federation,
        RootAuthorityCap,
        Federation,
        AccreditCap,
        add_statement,
        revoke_accreditation_to_attest,
        revoke_accreditation_to_accredit,
        create_accreditation_to_accredit,
        create_accreditation_to_attest,
        add_root_authority,
        revoke_root_authority,
        is_root_authority,
        revoke_statement
    },
    statement,
    statement_name::new_statement_name,
    statement_value::new_statement_value_number
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
    let statements = vector::empty();
    fed.create_accreditation_to_accredit(
        &accredit_cap,
        bob,
        statements,
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
    // Add a Statement

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let statements = vector::empty();
    fed.create_accreditation_to_attest(&accredit_cap, bob, statements, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was issued
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
    let statements = vector::empty();
    fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Issue permission to attest
    fed.create_accreditation_to_attest(&accredit_cap, bob, statements, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to attest
    let permission_id = fed
        .get_accreditations_to_attest(&bob)
        .accredited_statements()[0]
        .id()
        .uid_to_inner();
    fed.revoke_accreditation_to_attest(&accredit_cap, &bob, &permission_id, &clock, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to accredit
    let permission_id = fed
        .get_accreditations_to_accredit(&bob)
        .accredited_statements()[0]
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
#[expected_failure(abort_code = hierarchies::main::EStatementNotInFederation)]
fun test_create_accreditation_to_accredit_fails_for_nonexistent_statement() {
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
    fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, &clock, scenario.ctx());

    // Cleanup - this won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    new_id.delete();
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EStatementNotInFederation)]
fun test_create_accreditation_to_attest_fails_for_nonexistent_statement() {
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
    fed.create_accreditation_to_attest(&accredit_cap, bob, statements, &clock, scenario.ctx());

    // Cleanup - this won't be reached due to expected failure
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);
    clock.destroy_for_testing();
    new_id.delete();
    let _ = scenario.end();
}

#[test]
fun test_create_accreditation_to_accredit_succeeds_for_existing_statement() {
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
    fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, &clock, scenario.ctx());

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
fun test_create_accreditation_to_attest_succeeds_for_existing_statement() {
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
    fed.create_accreditation_to_attest(&accredit_cap, bob, statements, &clock, scenario.ctx());
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

    // First add two different statements to the federation
    let statement_name_1 = new_statement_name(utf8(b"role1"));
    let statement_name_2 = new_statement_name(utf8(b"role2"));
    let property_value = new_statement_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    fed.add_statement(&root_cap, statement_name_1, allowed_values, false, scenario.ctx());
    fed.add_statement(&root_cap, statement_name_2, allowed_values, false, scenario.ctx());
    scenario.next_tx(alice);

    // Create statements for permissions
    let statement_1 = statement::new_statement(
        statement_name_1,
        allowed_values,
        false,
        option::none(),
    );
    let statement_2 = statement::new_statement(
        statement_name_2,
        allowed_values,
        false,
        option::none(),
    );

    // Alice grants Charlie attestation rights for statement_2
    let charlie_statements = vector[statement_2];

    fed.create_accreditation_to_attest(
        &alice_accredit_cap,
        charlie.to_id(),
        charlie_statements,
        &clock,
        scenario.ctx(),
    );
    scenario.next_tx(alice);

    // Alice grants Bob accreditation rights only for statement_1 (not statement_2)
    let bob_statements = vector[statement_1];
    fed.create_accreditation_to_accredit(
        &alice_accredit_cap,
        bob.to_id(),
        bob_statements,
        &clock,
        scenario.ctx(),
    );
    scenario.next_tx(bob);

    // Bob receives his AccreditCap
    let bob_accredit_cap: AccreditCap = scenario.take_from_address(bob);

    // Get Charlie's attestation permission ID (for statement_2)
    let charlie_permission_id = fed
        .get_accreditations_to_attest(&charlie.to_id())
        .accredited_statements()[0]
        .id()
        .uid_to_inner();

    // Bob (who only has accreditation rights for statement_1, not statement_2)
    // tries to revoke Charlie's attestation rights for statement_2 - this should fail
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

    // Bob should be able to add a statement with his reinstated authority
    let statement_name = new_statement_name(utf8(b"test_statement"));
    let allowed_values = vec_set::empty();
    fed.add_statement(&bob_cap, statement_name, allowed_values, true, scenario.ctx());

    // Verify the statement was added
    assert!(fed.is_statement_in_federation(statement_name), 0);

    // Cleanup
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_to_address(bob, bob_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
#[expected_failure(abort_code = hierarchies::main::EStatementRevoked)]
fun test_create_accreditation_to_accredit_fails_for_revoked_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    let statement_name = new_statement_name(utf8(b"role"));
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(new_statement_value_number(1));
    fed.add_statement(&root_cap, statement_name, allowed_values, false, scenario.ctx());

    fed.revoke_statement(&root_cap, statement_name, &clock, scenario.ctx());

    let stmt = statement::new_statement(statement_name, vec_set::empty(), true, option::none());
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
#[expected_failure(abort_code = hierarchies::main::EStatementRevoked)]
fun test_create_accreditation_to_attest_fails_for_revoked_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    let statement_name = new_statement_name(utf8(b"role"));
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(new_statement_value_number(1));
    fed.add_statement(&root_cap, statement_name, allowed_values, false, scenario.ctx());

    fed.revoke_statement(&root_cap, statement_name, &clock, scenario.ctx());

    let stmt = statement::new_statement(statement_name, vec_set::empty(), true, option::none());
    fed.create_accreditation_to_attest(
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
    let statement_name = new_statement_name(utf8(b"test_statement"));
    let allowed_values = vec_set::empty();
    fed.add_statement(&transferred_cap, statement_name, allowed_values, true, scenario.ctx());

    // Cleanup - won't be reached due to expected failure
    test_scenario::return_to_address(alice, alice_cap);
    test_scenario::return_to_address(charlie, transferred_cap);
    test_scenario::return_shared(fed);
    let _ = scenario.end();
}

#[test]
fun test_validate_statement_fails_for_revoked_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // Add a statement to the federation
    let statement_name = new_statement_name(utf8(b"role"));
    let statement_value = new_statement_value_number(1);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(statement_value);
    fed.add_statement(&root_cap, statement_name, allowed_values, false, scenario.ctx());

    // Create accreditation for Bob to attest this statement
    let bob_id = @0x2.to_id();
    let stmt = statement::new_statement(statement_name, allowed_values, false, option::none());
    fed.create_accreditation_to_attest(&accredit_cap, bob_id, vector[stmt], &clock, scenario.ctx());

    // Initially validation should pass
    assert!(fed.validate_statement(&bob_id, statement_name, statement_value, &clock), 0);

    // Revoke the statement
    fed.revoke_statement(&root_cap, statement_name, &clock, scenario.ctx());

    // Now validation should fail because the statement is revoked
    assert!(!fed.validate_statement(&bob_id, statement_name, statement_value, &clock), 1);

    test_scenario::return_shared(fed);
    test_scenario::return_to_address(alice, root_cap);
    test_scenario::return_to_address(alice, accredit_cap);
    clock.destroy_for_testing();
    let _ = scenario.end();
}

#[test]
fun test_validate_statements_fails_for_revoked_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);
    let mut clock = clock::create_for_testing(scenario.ctx());
    clock.set_for_testing(1000);

    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let root_cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // Add two statements to the federation
    let statement_name_1 = new_statement_name(utf8(b"role1"));
    let statement_name_2 = new_statement_name(utf8(b"role2"));
    let statement_value_1 = new_statement_value_number(1);
    let statement_value_2 = new_statement_value_number(2);
    let mut allowed_values_1 = vec_set::empty();
    let mut allowed_values_2 = vec_set::empty();
    allowed_values_1.insert(statement_value_1);
    allowed_values_2.insert(statement_value_2);

    fed.add_statement(&root_cap, statement_name_1, allowed_values_1, false, scenario.ctx());
    fed.add_statement(&root_cap, statement_name_2, allowed_values_2, false, scenario.ctx());

    // Create accreditation for Bob to attest both statements
    let bob_id = @0x2.to_id();
    let stmt_1 = statement::new_statement(
        statement_name_1,
        allowed_values_1,
        false,
        option::none(),
    );
    let stmt_2 = statement::new_statement(
        statement_name_2,
        allowed_values_2,
        false,
        option::none(),
    );
    fed.create_accreditation_to_attest(
        &accredit_cap,
        bob_id,
        vector[stmt_1, stmt_2],
        &clock,
        scenario.ctx(),
    );

    // Create statement map for validation
    let mut statements = vec_map::empty();
    statements.insert(statement_name_1, statement_value_1);
    statements.insert(statement_name_2, statement_value_2);

    // Initially validation should pass
    assert!(fed.validate_statements(&bob_id, statements, &clock), 0);

    // Revoke the first statement
    fed.revoke_statement(&root_cap, statement_name_1, &clock, scenario.ctx());

    // Now validation should fail because one of the statements is revoked
    assert!(!fed.validate_statements(&bob_id, statements, &clock), 1);

    test_scenario::return_shared(fed);
    test_scenario::return_to_address(alice, root_cap);
    test_scenario::return_to_address(alice, accredit_cap);
    clock.destroy_for_testing();
    let _ = scenario.end();
}
