#[test_only]
module hierarchies::condition_tests;

use hierarchies::{statement_condition, statement_value};
use std::string;

#[test]
fun test_contains_at_beginning() {
    let condition = statement_condition::new_condition_contains(string::utf8(b"hello"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_starts_with_match() {
    let condition = statement_condition::new_condition_starts_with(string::utf8(b"hello"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_starts_with_no_match() {
    let condition = statement_condition::new_condition_starts_with(string::utf8(b"world"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(!statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_starts_with_wrong_type() {
    let condition = statement_condition::new_condition_starts_with(string::utf8(b"hello"));
    let value = statement_value::new_statement_value_number(123);

    assert!(!statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_contains_at_end() {
    let condition = statement_condition::new_condition_contains(string::utf8(b"world"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_ends_with_match() {
    let condition = statement_condition::new_condition_ends_with(string::utf8(b"world"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_ends_with_no_match() {
    let condition = statement_condition::new_condition_ends_with(string::utf8(b"hello"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(!statement_condition::condition_matches(&condition, &value), 0);
}



#[test]
fun test_contains_match() {
    let condition = statement_condition::new_condition_contains(string::utf8(b"lo wo"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_contains_no_match() {
    let condition = statement_condition::new_condition_contains(string::utf8(b"xyz"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(!statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_greater_than_match() {
    let condition = statement_condition::new_condition_greater_than(10);
    let value = statement_value::new_statement_value_number(15);

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_greater_than_no_match() {
    let condition = statement_condition::new_condition_greater_than(10);
    let value = statement_value::new_statement_value_number(5);

    assert!(!statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_greater_than_wrong_type() {
    let condition = statement_condition::new_condition_greater_than(10);
    let value = statement_value::new_statement_value_string(string::utf8(b"hello"));

    assert!(!statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_lower_than_match() {
    let condition = statement_condition::new_condition_lower_than(10);
    let value = statement_value::new_statement_value_number(5);

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_lower_than_no_match() {
    let condition = statement_condition::new_condition_lower_than(10);
    let value = statement_value::new_statement_value_number(15);

    assert!(!statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_string_shorter_than_condition() {
    let condition = statement_condition::new_condition_starts_with(string::utf8(b"hello world"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hi"));

    assert!(!statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_contains_single_character() {
    let condition = statement_condition::new_condition_contains(string::utf8(b"o"));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}

#[test]
fun test_contains_empty_string() {
    let condition = statement_condition::new_condition_contains(string::utf8(b""));
    let value = statement_value::new_statement_value_string(string::utf8(b"hello world"));

    assert!(statement_condition::condition_matches(&condition, &value), 0);
}


