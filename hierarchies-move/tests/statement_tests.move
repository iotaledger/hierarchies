module hierarchies::statement_tests;

use hierarchies::{
    statement::{Self, Statement},
    statement_name::{Self, StatementName},
    statement_value::{Self, StatementValue}
};
use iota::{vec_map, vec_set};
use std::string;

// ======= Helper Functions =======

fun create_test_statement_name_simple(name: vector<u8>): StatementName {
    statement_name::new_statement_name(string::utf8(name))
}

fun create_test_statement_value_simple(value: vector<u8>): StatementValue {
    statement_value::new_statement_value_string(string::utf8(value))
}

fun create_simple_statement(
    name: vector<u8>,
    allowed_value: vector<u8>,
    allow_any: bool,
): Statement {
    let statement_name = create_test_statement_name_simple(name);
    let mut value_set = vec_set::empty<StatementValue>();
    if (!allow_any) {
        vec_set::insert(&mut value_set, create_test_statement_value_simple(allowed_value));
    };

    statement::new_statement(statement_name, value_set, allow_any, option::none())
}

// ======= Timespan Tests =======

#[test]
fun test_new_timespan() {
    let timespan = statement::new_timespan(
        option::some(1000u64),
        option::some(2000u64),
    );

    assert!(statement::timestamp_matches(&timespan, 1500u64), 0);
}

#[test]
fun test_new_empty_timespan() {
    let timespan = statement::new_empty_timespan();
    // Empty timespan should match any timestamp
    assert!(statement::timestamp_matches(&timespan, 0u64), 0);
    assert!(statement::timestamp_matches(&timespan, 999999u64), 1);
}

#[test]
fun test_timestamp_matches_within_range() {
    let timespan = statement::new_timespan(
        option::some(1000u64),
        option::some(2000u64),
    );

    assert!(statement::timestamp_matches(&timespan, 1000u64), 0);
    assert!(statement::timestamp_matches(&timespan, 1500u64), 1);
    assert!(!statement::timestamp_matches(&timespan, 2000u64), 2);
}

#[test]
fun test_timestamp_matches_outside_range() {
    let timespan = statement::new_timespan(
        option::some(1000u64),
        option::some(2000u64),
    );

    assert!(!statement::timestamp_matches(&timespan, 999u64), 0);
    assert!(!statement::timestamp_matches(&timespan, 2001u64), 1);
}

// ======= Statement Tests =======

#[test]
fun test_new_statement() {
    let name = create_test_statement_name_simple(b"test");
    let mut values = vec_set::empty<StatementValue>();
    vec_set::insert(&mut values, create_test_statement_value_simple(b"value1"));

    let stmt = statement::new_statement(name, values, false, option::none());

    assert!(statement::statement_name(&stmt) == &name, 0);
    assert!(!statement::allowed_values(&stmt).is_empty(), 1);
}

#[test]
fun test_matches_value_allow_any() {
    let stmt = create_simple_statement(b"test", b"", true);
    let value = create_test_statement_value_simple(b"any_value");

    assert!(statement::matches_value(&stmt, &value, 1000u64), 0);
}

#[test]
fun test_matches_value_in_allowed_set() {
    let stmt = create_simple_statement(b"test", b"allowed_value", false);
    let value = create_test_statement_value_simple(b"allowed_value");

    assert!(statement::matches_value(&stmt, &value, 1000u64), 0);
}

#[test]
fun test_matches_value_not_in_allowed_set() {
    let stmt = create_simple_statement(b"test", b"allowed_value", false);
    let value = create_test_statement_value_simple(b"not_allowed");

    assert!(!statement::matches_value(&stmt, &value, 1000u64), 0);
}

#[test]
fun test_matches_name_value() {
    let stmt = create_simple_statement(b"test", b"value", false);
    let name = create_test_statement_name_simple(b"test");
    let value = create_test_statement_value_simple(b"value");

    assert!(statement::matches_name_value(&stmt, &name, &value, 1000u64), 0);
}

#[test]
fun test_revoke_statement() {
    let mut stmt = create_simple_statement(b"test", b"value", false);
    let name = create_test_statement_name_simple(b"test");
    let value = create_test_statement_value_simple(b"value");

    // Should match before revocation
    assert!(statement::matches_name_value(&stmt, &name, &value, 1000u64), 0);

    // Revoke the statement at time 1500
    statement::revoke(&mut stmt, 1500u64);

    // Should not match after revocation time
    assert!(!statement::matches_name_value(&stmt, &name, &value, 2000u64), 1);

    // Should still match before revocation time
    assert!(statement::matches_name_value(&stmt, &name, &value, 1000u64), 2);
}

// ======= Statements Tests =======

#[test]
fun test_new_statements() {
    let statements = statement::new_statements();
    let data = statement::data(&statements);

    assert!(vec_map::is_empty(data), 0);
    statement::destroy_statements(statements);
}

#[test]
fun test_add_statement() {
    let mut statements = statement::new_statements();
    let stmt = create_simple_statement(b"test", b"value", false);

    statement::add_statement(&mut statements, stmt);

    let data = statement::data(&statements);
    assert!(vec_map::size(data) == 1, 0);

    statement::destroy_statements(statements);
}

#[test]
fun test_to_map_of_statements() {
    let stmt1 = create_simple_statement(b"test1", b"value1", false);
    let stmt2 = create_simple_statement(b"test2", b"value2", false);

    let statements_vec = vector[stmt1, stmt2];
    let map = statement::to_map_of_statements(statements_vec);

    assert!(vec_map::size(&map) == 2, 0);
    assert!(vec_map::contains(&map, &create_test_statement_name_simple(b"test1")), 1);
    assert!(vec_map::contains(&map, &create_test_statement_name_simple(b"test2")), 2);
}
