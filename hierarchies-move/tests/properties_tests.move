module hierarchies::property_tests;

use hierarchies::{
    property::{Self, FederationProperty},
    property_name::{Self, PropertyName},
    property_value::{Self, PropertyValue}
};
use iota::{vec_map, vec_set};
use std::string;

// ======= Helper Functions =======

fun create_test_property_name_simple(name: vector<u8>): PropertyName {
    property_name::new_property_name(string::utf8(name))
}

fun create_test_property_value_simple(value: vector<u8>): PropertyValue {
    property_value::new_property_value_string(string::utf8(value))
}

fun create_simple_property(
    name: vector<u8>,
    allowed_value: vector<u8>,
    allow_any: bool,
): FederationProperty {
    let property_name = create_test_property_name_simple(name);
    let mut value_set = vec_set::empty<PropertyValue>();
    if (!allow_any) {
        vec_set::insert(&mut value_set, create_test_property_value_simple(allowed_value));
    };

    property::new_property(property_name, value_set, allow_any, option::none())
}

// ======= Timespan Tests =======

#[test]
fun test_new_timespan() {
    let timespan = property::new_timespan(
        option::some(1000u64),
        option::some(2000u64),
    );

    assert!(property::timestamp_matches(&timespan, 1500u64), 0);
}

#[test]
fun test_new_empty_timespan() {
    let timespan = property::new_empty_timespan();
    // Empty timespan should match any timestamp
    assert!(property::timestamp_matches(&timespan, 0u64), 0);
    assert!(property::timestamp_matches(&timespan, 999999u64), 1);
}

#[test]
fun test_timestamp_matches_within_range() {
    let timespan = property::new_timespan(
        option::some(1000u64),
        option::some(2000u64),
    );

    assert!(property::timestamp_matches(&timespan, 1000u64), 0);
    assert!(property::timestamp_matches(&timespan, 1500u64), 1);
    assert!(!property::timestamp_matches(&timespan, 2000u64), 2);
}

#[test]
fun test_timestamp_matches_outside_range() {
    let timespan = property::new_timespan(
        option::some(1000u64),
        option::some(2000u64),
    );

    assert!(!property::timestamp_matches(&timespan, 999u64), 0);
    assert!(!property::timestamp_matches(&timespan, 2001u64), 1);
}

// ======= Property Tests =======

#[test]
fun test_new_property() {
    let name = create_test_property_name_simple(b"test");
    let mut values = vec_set::empty<PropertyValue>();
    vec_set::insert(&mut values, create_test_property_value_simple(b"value1"));

    let property = property::new_property(name, values, false, option::none());

    assert!(property::property_name(&property) == &name, 0);
    assert!(!property::allowed_values(&property).is_empty(), 1);
}

#[test]
fun test_matches_value_allow_any() {
    let property = create_simple_property(b"test", b"", true);
    let value = create_test_property_value_simple(b"any_value");

    assert!(property::matches_value(&property, &value, 1000u64), 0);
}

#[test]
fun test_matches_value_in_allowed_set() {
    let property = create_simple_property(b"test", b"allowed_value", false);
    let value = create_test_property_value_simple(b"allowed_value");

    assert!(property::matches_value(&property, &value, 1000u64), 0);
}

#[test]
fun test_matches_value_not_in_allowed_set() {
    let property = create_simple_property(b"test", b"allowed_value", false);
    let value = create_test_property_value_simple(b"not_allowed");

    assert!(!property::matches_value(&property, &value, 1000u64), 0);
}

#[test]
fun test_matches_name_value() {
    let property = create_simple_property(b"test", b"value", false);
    let name = create_test_property_name_simple(b"test");
    let value = create_test_property_value_simple(b"value");

    assert!(property::matches_name_value(&property, &name, &value, 1000u64), 0);
}

#[test]
fun test_revoke_property() {
    let mut property = create_simple_property(b"test", b"value", false);
    let name = create_test_property_name_simple(b"test");
    let value = create_test_property_value_simple(b"value");

    // Should match before revocation
    assert!(property::matches_name_value(&property, &name, &value, 1000u64), 0);

    // Revoke the property at time 1500
    property::revoke(&mut property, 1500u64);

    // Should not match after revocation time
    assert!(!property::matches_name_value(&property, &name, &value, 2000u64), 1);

    // Should still match before revocation time
    assert!(property::matches_name_value(&property, &name, &value, 1000u64), 2);
}

// ======= Properties Tests =======

#[test]
fun test_new_properties() {
    let properties = property::new_properties();
    let data = property::data(&properties);

    assert!(vec_map::is_empty(data), 0);
    property::destroy_properties(properties);
}

#[test]
fun test_add_property() {
    let mut properties = property::new_properties();
    let property = create_simple_property(b"test", b"value", false);

    property::add_property(&mut properties, property);

    let data = property::data(&properties);
    assert!(vec_map::size(data) == 1, 0);

    property::destroy_properties(properties);
}

#[test]
fun test_to_map_of_properties() {
    let property1 = create_simple_property(b"test1", b"value1", false);
    let property2 = create_simple_property(b"test2", b"value2", false);

    let properties_vec = vector[property1, property2];
    let map = property::to_map_of_properties(properties_vec);

    assert!(vec_map::size(&map) == 2, 0);
    assert!(vec_map::contains(&map, &create_test_property_name_simple(b"test1")), 1);
    assert!(vec_map::contains(&map, &create_test_property_name_simple(b"test2")), 2);
}
