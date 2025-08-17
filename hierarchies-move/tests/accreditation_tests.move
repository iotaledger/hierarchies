#[test_only]
module hierarchies::accreditation_tests;

use hierarchies::{
    accreditation::{Self, Accreditation},
    property::{Self, FederationProperty},
    property_shape,
    property_name,
    property_value::{Self, PropertyValue}
};
use iota::{test_scenario::{Self, Scenario}, vec_map, vec_set::{Self, VecSet}};
use std::string;

fun create_test_property_simple(name: vector<u8>, value: vector<u8>): FederationProperty {
    let property_name = property_name::new_property_name(string::utf8(name));
    let mut value_set = vec_set::empty();
    vec_set::insert(
        &mut value_set,
        property_value::new_property_value_string(string::utf8(value)),
    );
    property::new_property(property_name, value_set, false, option::none())
}

fun test_accreditation_creation(): (Scenario, Accreditation) {
    let mut scenario = test_scenario::begin(@0x1);
    let property = create_test_property_simple(b"role", b"admin");
    let accreditation = accreditation::new_accreditation(vector[property], scenario.ctx());
    (scenario, accreditation)
}

#[test]
fun test_new_empty_accreditations() {
    let accreditations = accreditation::new_empty_accreditations();
    assert!(accreditation::accredited_properties(&accreditations).is_empty(), 0);
    accreditation::destroy_accreditations(accreditations);
}

#[test]
fun test_new_accreditations_with_properties() {
    let (scenario, accreditation) = test_accreditation_creation();
    let accreditations = accreditation::new_accreditations(vector[accreditation]);
    assert!(accreditation::accredited_properties(&accreditations).length() == 1, 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_add_accreditation() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();

    accreditation::add_accreditation(&mut accreditations, accreditation);
    assert!(accreditation::accredited_properties(&accreditations).length() == 1, 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_accredited_properties_getter() {
    let (mut scenario, accreditation1) = test_accreditation_creation();
    let property2 = create_test_property_simple(b"department", b"engineering");
    let accreditation2 = accreditation::new_accreditation(vector[property2], scenario.ctx());

    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation1);
    accreditation::add_accreditation(&mut accreditations, accreditation2);

    let properties = accreditation::accredited_properties(&accreditations);
    assert!(properties.length() == 2, 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_allowed_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let name = property_name::new_property_name(string::utf8(b"role"));
    let value = property_value::new_property_value_string(string::utf8(b"admin"));

    assert!(accreditation::is_property_allowed(&accreditations, &name, &value, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_allowed_no_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let name = property_name::new_property_name(string::utf8(b"role"));
    let value = property_value::new_property_value_string(string::utf8(b"user"));

    assert!(!accreditation::is_property_allowed(&accreditations, &name, &value, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_properties_allowed_all_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let mut properties = vec_map::empty();
    vec_map::insert(
        &mut properties,
        property_name::new_property_name(string::utf8(b"role")),
        property_value::new_property_value_string(string::utf8(b"admin")),
    );

    assert!(accreditation::are_properties_allowed(&accreditations, &properties, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_properties_allowed_some_fail() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let mut properties = vec_map::empty();
    vec_map::insert(
        &mut properties,
        property_name::new_property_name(string::utf8(b"role")),
        property_value::new_property_value_string(string::utf8(b"user")), // Not allowed
    );

    assert!(!accreditation::are_properties_allowed(&accreditations, &properties, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_properties_allowed_empty() {
    let accreditations = accreditation::new_empty_accreditations();
    let properties = vec_map::empty();

    assert!(accreditation::are_properties_allowed(&accreditations, &properties, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
}

#[test]
fun test_is_property_compliant_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let property = create_test_property_simple(b"role", b"admin");
    assert!(accreditation::is_property_compliant(&accreditations, &property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_properties_compliant_all_match() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let properties = vector[create_test_property_simple(b"role", b"admin")];
    assert!(accreditation::are_properties_compliant(&accreditations, &properties, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_are_properties_compliant_empty() {
    let accreditations = accreditation::new_empty_accreditations();
    let properties = vector[];

    assert!(accreditation::are_properties_compliant(&accreditations, &properties, 1000), 0);

    accreditation::destroy_accreditations(accreditations);
}

#[test]
fun test_new_accreditation() {
    let (mut scenario, accreditation) = test_accreditation_creation();

    assert!(accreditation::accredited_by(&accreditation) == scenario.ctx().sender().to_string(), 0);
    assert!(!vec_map::is_empty(accreditation::properties(&accreditation)), 1);
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
fun test_accreditation_properties_getter() {
    let (scenario, accreditation) = test_accreditation_creation();

    let properties = accreditation::properties(&accreditation);
    assert!(vec_map::size(properties) == 1, 0);

    let role_name = property_name::new_property_name(string::utf8(b"role"));
    assert!(vec_map::contains(properties, &role_name), 1);
    accreditation::destroy_accreditation(accreditation);
    scenario.end();
}

#[test]
fun test_find_accreditation_by_id() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    let id = accreditation::id(&accreditation).to_inner();

    accreditation::add_accreditation(&mut accreditations, accreditation);

    let found_idx = accreditation::find_accredited_property_id(&accreditations, &id);
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
    let found_idx = accreditation::find_accredited_property_id(
        &accreditations,
        non_existent_id.as_inner(),
    );
    assert!(found_idx.is_none(), 0);
    accreditation::destroy_accreditations(accreditations);
    iota::object::delete(non_existent_id);
    scenario.end();
}

#[test]
fun test_remove_accredited_property() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    let id = accreditation::id(&accreditation).to_inner();

    accreditation::add_accreditation(&mut accreditations, accreditation);
    assert!(accreditation::accredited_properties(&accreditations).length() == 1, 0);

    accreditation::remove_accredited_property(&mut accreditations, &id);
    assert!(accreditation::accredited_properties(&accreditations).length() == 0, 1);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_remove_accredited_property_not_found() {
    let (mut scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    let non_existent_id = iota::object::new(scenario.ctx());
    accreditation::remove_accredited_property(&mut accreditations, non_existent_id.as_inner());

    // Should still have 1 accreditation since removal failed
    assert!(accreditation::accredited_properties(&accreditations).length() == 1, 0);
    accreditation::destroy_accreditations(accreditations);
    iota::object::delete(non_existent_id);
    scenario.end();
}

// ===== Tests for is_statement_compliant function =====

fun create_test_property_with_multiple_values(
    name: vector<u8>,
    values: vector<vector<u8>>,
): FederationProperty {
    let property_name = property_name::new_property_name(string::utf8(name));
    let mut value_set = vec_set::empty();
    let mut idx = 0;
    while (idx < values.length()) {
        vec_set::insert(
            &mut value_set,
            property_value::new_property_value_string(string::utf8(values[idx])),
        );
        idx = idx + 1;
    };
    property::new_property(property_name, value_set, false, option::none())
}

fun create_test_property_with_condition(
    name: vector<u8>,
    condition: property_shape::PropertyShape,
): FederationProperty {
    let property_name = property_name::new_property_name(string::utf8(name));
    let value_set: VecSet<PropertyValue> = vec_set::empty();
    property::new_property(property_name, value_set, false, option::some(condition))
}

#[test]
fun test_is_property_compliant_multiple_values_all_covered() {
    let mut scenario = test_scenario::begin(@0x1);

    // Create accreditation with multiple values
    let property = create_test_property_with_multiple_values(
        b"role",
        vector[b"admin", b"user", b"guest"],
    );
    let accreditation = accreditation::new_accreditation(vector[property], scenario.ctx());
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    // Create a statement that requires all values
    let test_property = create_test_property_with_multiple_values(
        b"role",
        vector[b"admin", b"user", b"guest"],
    );

    assert!(accreditation::is_property_compliant(&accreditations, &test_property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_compliant_multiple_values_partial_covered() {
    let mut scenario = test_scenario::begin(@0x1);

    // Create accreditation with multiple values
    let property = create_test_property_with_multiple_values(b"role", vector[b"admin", b"user"]);
    let accreditation = accreditation::new_accreditation(vector[property], scenario.ctx());
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    // Create a statement that requires more values than available
    let test_property = create_test_property_with_multiple_values(
        b"role",
        vector[b"admin", b"user", b"guest"],
    );

    assert!(!accreditation::is_property_compliant(&accreditations, &test_property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_compliant_multiple_values_subset_covered() {
    let mut scenario = test_scenario::begin(@0x1);

    // Create accreditation with multiple values
    let property = create_test_property_with_multiple_values(
        b"role",
        vector[b"admin", b"user", b"guest"],
    );
    let accreditation = accreditation::new_accreditation(vector[property], scenario.ctx());
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    // Create a statement that requires only a subset
    let test_property = create_test_property_with_multiple_values(
        b"role",
        vector[b"admin", b"user"],
    );

    assert!(accreditation::is_property_compliant(&accreditations, &test_property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_compliant_with_condition() {
    let mut scenario = test_scenario::begin(@0x1);

    // Create accreditation with condition
    let condition = property_shape::new_property_shape_contains(string::utf8(b"admin"));
    let property = create_test_property_with_condition(b"role", condition);
    let accreditation = accreditation::new_accreditation(vector[property], scenario.ctx());
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    // Create a statement that matches the condition
    let test_property = create_test_property_simple(b"role", b"admin_user");

    assert!(accreditation::is_property_compliant(&accreditations, &test_property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_compliant_with_condition_no_match() {
    let mut scenario = test_scenario::begin(@0x1);

    // Create accreditation with condition
    let condition = property_shape::new_property_shape_contains(string::utf8(b"admin"));
    let property = create_test_property_with_condition(b"role", condition);
    let accreditation = accreditation::new_accreditation(vector[property], scenario.ctx());
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    // Create a statement that doesn't match the condition
    let test_property = create_test_property_simple(b"role", b"user");

    assert!(!accreditation::is_property_compliant(&accreditations, &test_property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_compliant_empty_property_values() {
    let (scenario, accreditation) = test_accreditation_creation();
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    // Create a statement with no values
    let property_name = property_name::new_property_name(string::utf8(b"role"));
    let empty_value_set = vec_set::empty();
    let test_property = property::new_property(
        property_name,
        empty_value_set,
        false,
        option::none(),
    );

    assert!(accreditation::is_property_compliant(&accreditations, &test_property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_compliant_name_matching_fix() {
    let mut scenario = test_scenario::begin(@0x1);

    // Create accreditation with hierarchical name "role.admin"
    let role_admin_name = property_name::new_property_name(string::utf8(b"role.admin"));
    let mut value_set = vec_set::empty();
    vec_set::insert(
        &mut value_set,
        property_value::new_property_value_string(string::utf8(b"superuser")),
    );
    let property = property::new_property(role_admin_name, value_set, false, option::none());
    let accreditation = accreditation::new_accreditation(vector[property], scenario.ctx());
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    // Create a statement with name "role" (should not match due to name matching fix)
    let test_property = create_test_property_simple(b"role", b"superuser");

    assert!(!accreditation::is_property_compliant(&accreditations, &test_property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}

#[test]
fun test_is_property_compliant_name_matching_success() {
    let mut scenario = test_scenario::begin(@0x1);

    // Create accreditation with hierarchical name "role.admin"
    let role_admin_name = property_name::new_property_name(string::utf8(b"role.admin"));
    let mut value_set = vec_set::empty();
    vec_set::insert(
        &mut value_set,
        property_value::new_property_value_string(string::utf8(b"superuser")),
    );
    let property = property::new_property(role_admin_name, value_set, false, option::none());
    let accreditation = accreditation::new_accreditation(vector[property], scenario.ctx());
    let mut accreditations = accreditation::new_empty_accreditations();
    accreditation::add_accreditation(&mut accreditations, accreditation);

    // Create a statement with name "role.admin" (should match)
    let test_property = create_test_property_simple(b"role.admin", b"superuser");

    assert!(accreditation::is_property_compliant(&accreditations, &test_property, 1000), 0);
    accreditation::destroy_accreditations(accreditations);
    scenario.end();
}
