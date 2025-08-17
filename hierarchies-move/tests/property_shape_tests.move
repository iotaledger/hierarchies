#[test_only]
module hierarchies::property_shape_tests;

use hierarchies::{property_shape, property_value};
use std::string;

#[test]
fun test_contains_at_beginning() {
    let condition = property_shape::new_property_shape_contains(string::utf8(b"hello"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_starts_with_match() {
    let condition = property_shape::new_property_shape_starts_with(string::utf8(b"hello"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_starts_with_no_match() {
    let condition = property_shape::new_property_shape_starts_with(string::utf8(b"world"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(!property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_starts_with_wrong_type() {
    let condition = property_shape::new_property_shape_starts_with(string::utf8(b"hello"));
    let value = property_value::new_property_value_number(123);

    assert!(!property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_contains_at_end() {
    let condition = property_shape::new_property_shape_contains(string::utf8(b"world"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_ends_with_match() {
    let condition = property_shape::new_property_shape_ends_with(string::utf8(b"world"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_ends_with_no_match() {
    let condition = property_shape::new_property_shape_ends_with(string::utf8(b"hello"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(!property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_contains_match() {
    let condition = property_shape::new_property_shape_contains(string::utf8(b"lo wo"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_contains_no_match() {
    let condition = property_shape::new_property_shape_contains(string::utf8(b"xyz"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(!property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_greater_than_match() {
    let condition = property_shape::new_property_shape_greater_than(10);
    let value = property_value::new_property_value_number(15);

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_greater_than_no_match() {
    let condition = property_shape::new_property_shape_greater_than(10);
    let value = property_value::new_property_value_number(5);

    assert!(!property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_greater_than_wrong_type() {
    let condition = property_shape::new_property_shape_greater_than(10);
    let value = property_value::new_property_value_string(string::utf8(b"hello"));

    assert!(!property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_lower_than_match() {
    let condition = property_shape::new_property_shape_lower_than(10);
    let value = property_value::new_property_value_number(5);

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_lower_than_no_match() {
    let condition = property_shape::new_property_shape_lower_than(10);
    let value = property_value::new_property_value_number(15);

    assert!(!property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_string_shorter_than_condition() {
    let condition = property_shape::new_property_shape_starts_with(string::utf8(b"hello world"));
    let value = property_value::new_property_value_string(string::utf8(b"hi"));

    assert!(!property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_contains_single_character() {
    let condition = property_shape::new_property_shape_contains(string::utf8(b"o"));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}

#[test]
fun test_contains_empty_string() {
    let condition = property_shape::new_property_shape_contains(string::utf8(b""));
    let value = property_value::new_property_value_string(string::utf8(b"hello world"));

    assert!(property_shape::property_shape_matches(&condition, &value), 0);
}
