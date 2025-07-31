#[test_only]
module hierarchies::accreditation_tests;

use iota::{test_scenario::{Self, Scenario}, vec_map, vec_set};
use hierarchies::{
    accreditation::{Self, Accreditation},
    statement::{Self, Statement},
    statement_name,
    statement_value
};
use std::string;

fun create_test_statement_simple(name: vector<u8>, value: vector<u8>): Statement {
    let statement_name = statement_name::new_statement_name(string::utf8(name));
    let mut value_set = vec_set::empty();
    vec_set::insert(
        &mut value_set,
        statement_value::new_statement_value_string(string::utf8(value)),
    );
    statement::new_statement(statement_name, value_set, false, option::none())
}

fun test_accreditation_creation(): (Scenario, Accreditation) {
    let mut scenario = test_scenario::begin(@0x1);
    let stmt = create_test_statement_simple(b"role", b"admin");
    let accreditation = accreditation::new_accreditation(vector[stmt], scenario.ctx());
    (scenario, accreditation)
}

#[test]
fun test_new_empty_accreditations() {
    let accreditations = accreditation::new_empty_accreditations();
    assert!(accreditation::accredited_statements(&accreditations).is_empty(), 0);
    accreditation::destroy_accreditations(accreditations);
}

#[test]
fun test_new_accreditations_with_statements() {
    let (scenario, accreditation) = test_accreditation_creation();
    let accreditations = accreditation::new_accreditations(vector[accreditation]);
    assert!(accreditation::accredited_statements(&accreditations).length() == 1, 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_add_accreditation() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();

    accreditation::add_accreditation(&mut accreditations, accreditation);
    assert!(accreditation::accredited_statements(&accreditations).length() == 1, 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_add_accredited_statement() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();

    accreditation::add_accredited_statement(&mut accreditations, accreditation);
    assert!(accreditation::accredited_statements(&accreditations).length() == 1, 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_accredited_statements_getter() {
    let (mut scenario, accreditation1) = test_accreditation_creation();
    let stmt2 = create_test_statement_simple(b"department", b"engineering");
    let accreditation2 = accreditation::new_accreditation(vector[stmt2], scenario.ctx());

    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation1);
    accreditation::add_accreditation(&mut accreditations, accreditation2);

    let statements = accreditation::accredited_statements(&accreditations);
    assert!(statements.length() == 2, 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_statement_allowed_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let name = statement_name::new_statement_name(string::utf8(b"role"));
    let value = statement_value::new_statement_value_string(string::utf8(b"admin"));

    assert!(accreditation::is_statement_allowed(&accreditations, &name, &value, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_statement_allowed_no_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let name = statement_name::new_statement_name(string::utf8(b"role"));
    let value = statement_value::new_statement_value_string(string::utf8(b"user"));

    assert!(!accreditation::is_statement_allowed(&accreditations, &name, &value, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_statements_allowed_all_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let mut statements = vec_map::empty();
    vec_map::insert(
        &mut statements,
        statement_name::new_statement_name(string::utf8(b"role")),
        statement_value::new_statement_value_string(string::utf8(b"admin")),
    );

    assert!(accreditation::are_statements_allowed(&accreditations, &statements, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_statements_allowed_some_fail() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let mut statements = vec_map::empty();
    vec_map::insert(
        &mut statements,
        statement_name::new_statement_name(string::utf8(b"role")),
        statement_value::new_statement_value_string(string::utf8(b"user")), // Not allowed
    );

    assert!(!accreditation::are_statements_allowed(&accreditations, &statements, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_statements_allowed_empty() {
    let accreditations = accreditation::new_empty_accreditations();
    let statements = vec_map::empty();

    assert!(accreditation::are_statements_allowed(&accreditations, &statements, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
}

#[test]
fun test_is_statement_compliant_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let stmt = create_test_statement_simple(b"role", b"admin");
    assert!(accreditation::is_statement_compliant(&accreditations, &stmt, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_statements_compliant_all_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let statements = vector[create_test_statement_simple(b"role", b"admin")];
    assert!(accreditation::are_statements_compliant(&accreditations, &statements, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_statements_compliant_empty() {
    let accreditations = accreditation::new_empty_accreditations();
    let statements = vector[];

    assert!(accreditation::are_statements_compliant(&accreditations, &statements, 1000), 0);

    accreditation::destroy_accreditations(accreditations);
}

#[test]
fun test_new_accreditation() {
    let (mut scenario, accreditation) = test_accreditation_creation();

    assert!(accreditation::accredited_by(&accreditation) == scenario.ctx().sender().to_string(), 0);
    assert!(!vec_map::is_empty(accreditation::statements(&accreditation)), 1);
    accreditation::destroy_accreditation(accreditation);
    scenario.end();
}

#[test]
fun test_accreditation_accredited_by_getter() {
    let (mut scenario, accreditation) = test_accreditation_creation();

    let accredited_by = accreditation::accredited_by(&accreditation);
    assert!(accredited_by == scenario.ctx().sender().to_string(), 0);
    accreditation::destroy_accreditation(accreditation);
    scenario.end();
}

#[test]
fun test_accreditation_statements_getter() {
    let (scenario, accreditation) = test_accreditation_creation();

    let statements = accreditation::statements(&accreditation);
    assert!(vec_map::size(statements) == 1, 0);

    let role_name = statement_name::new_statement_name(string::utf8(b"role"));
    assert!(vec_map::contains(statements, &role_name), 1);
    accreditation::destroy_accreditation(accreditation);
    scenario.end();
}

#[test]
fun test_find_accreditation_by_id() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    let id = accreditation::id(&accreditation).to_inner();

    accreditation::add_accreditation(&mut accreditations, accreditation);

    let found_idx = accreditation::find_accredited_statement_id(&accreditations, &id);
    assert!(found_idx.is_some(), 0);
    assert!(*found_idx.borrow() == 0, 1);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_find_accreditation_by_id_not_found() {
    let (mut scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let non_existent_id = iota::object::new(scenario.ctx());
    let found_idx = accreditation::find_accredited_statement_id(
        &accreditations,
        non_existent_id.as_inner(),
    );
    assert!(found_idx.is_none(), 0);
    accreditation::destroy_accreditations(accreditations);
    iota::object::delete(non_existent_id);
    scenario.end();
}

#[test]
fun test_remove_accredited_statement() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    let id = accreditation::id(&accreditation).to_inner();

    accreditation::add_accreditation(&mut accreditations, accreditation);
    assert!(accreditation::accredited_statements(&accreditations).length() == 1, 0);

    accreditation::remove_accredited_statement(&mut accreditations, &id);
    assert!(accreditation::accredited_statements(&accreditations).length() == 0, 1);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_remove_accredited_statement_not_found() {
    let (mut scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let non_existent_id = iota::object::new(scenario.ctx());
    accreditation::remove_accredited_statement(&mut accreditations, non_existent_id.as_inner());

    // Should still have 1 accreditation since removal failed
    assert!(accreditation::accredited_statements(&accreditations).length() == 1, 0);
    accreditation::destroy_accreditations(accreditations);
    iota::object::delete(non_existent_id);
    scenario.end();
}
