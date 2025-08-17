module hierarchies::property;

use hierarchies::{
    property_shape::PropertyShape,
    property_name::PropertyName,
    property_value::PropertyValue
};
use iota::{vec_map::{Self, VecMap}, vec_set::VecSet};

// Properties is a struct that contains a map of StatementName to Statement
public struct Properties has store {
    data: VecMap<PropertyName, Property>,
}

// The evaluation order: allow_any => condition => allowed_values
// The evaluation order is determined by the possible size of the set of values
// that match the condition.
public struct Property has copy, drop, store {
    property_name: PropertyName,
    // allow only values that are in the set
    allowed_values: VecSet<PropertyValue>,
    // Allow only values that match the condition.
    condition: Option<PropertyShape>,
    // If true, the statement is not applied, any value is allowed
    allow_any: bool,
    // The time span of the statement
    timespan: Timespan,
}

/// Creates a new Statement
public fun new_property(
    property_name: PropertyName,
    allowed_values: VecSet<PropertyValue>,
    allow_any: bool,
    condition: Option<PropertyShape>,
): Property {
    Property {
        property_name,
        allowed_values,
        condition,
        allow_any,
        timespan: new_empty_timespan(),
    }
}

public(package) fun new_properties(): Properties {
    Properties {
        data: vec_map::empty(),
    }
}

public(package) fun data(self: &Properties): &VecMap<PropertyName, Property> {
    &self.data
}

public(package) fun data_mut(self: &mut Properties): &mut VecMap<PropertyName, Property> {
    &mut self.data
}

public(package) fun add_property(self: &mut Properties, property: Property) {
    let name = property.property_name;
    self.data.insert(name, property)
}

public(package) fun allowed_values(self: &Property): &VecSet<PropertyValue> {
    &self.allowed_values
}

public(package) fun property_name(self: &Property): &PropertyName {
    &self.property_name
}

public(package) fun matches_name_value(
    self: &Property,
    name: &PropertyName,
    value: &PropertyValue,
    current_time_ms: u64,
): bool {
    self.matches_name(name) && self.matches_value(value, current_time_ms)
}

public(package) fun matches_name(self: &Property, name: &PropertyName): bool {
    // considering the statement name is a.b.c
    // the allowed name should be equal a.b.c or longer
    let len_statement = self.property_name.names().length();
    let len_names = name.names().length();

    // if it's longer than the name, it's not possible to match
    if (len_statement > len_names) {
        return false
    };

    let mut idx = 0;
    while (idx < len_statement) {
        if (self.property_name.names()[idx] != name.names()[idx]) {
            // if you have a.b.c and a.b.d, it is not possible to match
            return false
        };
        idx = idx + 1;
    };

    true
}

public(package) fun matches_value(
    self: &Property,
    value: &PropertyValue,
    current_time_ms: u64,
): bool {
    if (!self.timespan.timestamp_matches(current_time_ms)) {
        return false
    };

    if (self.allow_any) {
        return true
    };
    if (self.condition.is_some()) {
        if (self.condition.borrow().property_shape_matches(value)) {
            return true
        }
    };
    self.allowed_values.contains(value)
}

public(package) fun revoke(self: &mut Property, valid_to_ms: u64) {
    self.timespan.valid_until_ms = option::some(valid_to_ms)
}

/// Checks if a property is valid (not revoked) at the given time
public(package) fun is_valid_at_time(self: &Property, current_time_ms: u64): bool {
    self.timespan.timestamp_matches(current_time_ms)
}

public(package) fun to_map_of_properties(
    properties: vector<Property>,
): VecMap<PropertyName, Property> {
    let mut idx = 0;
    let mut map: VecMap<PropertyName, Property> = vec_map::empty();
    while (idx < properties.length()) {
        let property = properties[idx];
        map.insert(*property.property_name(), property);
        idx = idx + 1;
    };
    return map
}

/// Represents a time statement. The valid_from_ms and valid_until_ms are
/// optional, if they are not set, the property is valid for all time.
public struct Timespan has copy, drop, store {
    valid_from_ms: Option<u64>,
    valid_until_ms: Option<u64>,
}

public(package) fun new_timespan(
    valid_from_ms: Option<u64>,
    valid_until_ms: Option<u64>,
): Timespan {
    Timespan {
        valid_from_ms,
        valid_until_ms,
    }
}

public(package) fun new_empty_timespan(): Timespan {
    Timespan {
        valid_from_ms: option::none(),
        valid_until_ms: option::none(),
    }
}

public(package) fun timestamp_matches(self: &Timespan, now_ms: u64): bool {
    if (self.valid_from_ms.is_some() && *self.valid_from_ms.borrow() > now_ms) {
        return false
    };
    if (self.valid_until_ms.is_some() && *self.valid_until_ms.borrow() <= now_ms) {
        return false
    };
    true
}

// ===== Test-only Functions =====
#[test_only]
public(package) fun destroy_properties(properties: Properties) {
    let Properties { data: _ } = properties;
}
