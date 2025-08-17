module hierarchies::property_shape;

use hierarchies::property_value::PropertyValue;
use std::string::String;

/// PropertyShape defines the shape of a property.
public enum PropertyShape has copy, drop, store {
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    GreaterThan(u64),
    LowerThan(u64),
}

/// Creates a new PropertyShape that checks if the value starts with the given text.
public fun new_property_shape_starts_with(text: String): PropertyShape {
    PropertyShape::StartsWith(text)
}

/// Creates a new PropertyShape that checks if the value ends with the given text.
public fun new_property_shape_ends_with(text: String): PropertyShape {
    PropertyShape::EndsWith(text)
}

/// Creates a new PropertyShape that checks if the value contains the given text.
public fun new_property_shape_contains(text: String): PropertyShape {
    PropertyShape::Contains(text)
}

/// Creates a new PropertyShape that checks if the value is greater than the given number.
public fun new_property_shape_greater_than(value: u64): PropertyShape {
    PropertyShape::GreaterThan(value)
}

/// Creates a new PropertyShape that checks if the value is lower than the given number.
public fun new_property_shape_lower_than(value: u64): PropertyShape {
    PropertyShape::LowerThan(value)
}

/// Checks if the condition matches the value.
public fun property_shape_matches(self: &PropertyShape, value: &PropertyValue): bool {
    match (self) {
        PropertyShape::StartsWith(ref_string) => {
            let maybe_value_string = value.as_string();
            if (maybe_value_string.is_none()) {
                return false
            };
            let value_string = maybe_value_string.borrow();
            if (value_string.length() < ref_string.length()) {
                return false
            };
            return value_string.index_of(ref_string) == 0
        },
        PropertyShape::EndsWith(ref_string) => {
            let maybe_value_string = value.as_string();
            if (maybe_value_string.is_none()) {
                return false
            };
            let value_string = maybe_value_string.borrow();
            if (value_string.length() < ref_string.length()) {
                return false
            };
            return value_string.index_of(ref_string) == value_string.length() - ref_string.length()
        },
        PropertyShape::Contains(ref_string) => {
            let maybe_value_string = value.as_string();
            if (maybe_value_string.is_none()) {
                return false
            };
            let value_string = maybe_value_string.borrow();
            if (value_string.length() < ref_string.length()) {
                return false
            };
            let index = value_string.index_of(ref_string);
            return index < value_string.length()
        },
        PropertyShape::GreaterThan(ref_value) => {
            let maybe_value_number = value.as_number();
            if (maybe_value_number.is_none()) {
                return false
            };
            let value_number = maybe_value_number.borrow();
            return *value_number > *ref_value
        },
        PropertyShape::LowerThan(ref_value) => {
            let maybe_value_number = value.as_number();
            if (maybe_value_number.is_none()) {
                return false
            };
            let value_number = maybe_value_number.borrow();
            return *value_number < *ref_value
        },
    }
}
