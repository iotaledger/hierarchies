module hierarchies::property_value;

use std::string::String;

/// PropertyValue can be a String or a Number.
public enum PropertyValue has copy, drop, store {
    String(String),
    Number(u64),
}

/// Creates a new StatementValue from a String.
public fun new_property_value_string(v: String): PropertyValue {
    PropertyValue::String(v)
}

/// Creates a new StatementValue from a u64 number.
public fun new_property_value_number(v: u64): PropertyValue {
    PropertyValue::Number(v)
}

public(package) fun as_string(self: &PropertyValue): Option<String> {
    match (self) {
        PropertyValue::String(text) => option::some(*text),
        PropertyValue::Number(_) => option::none(),
    }
}

public(package) fun as_number(self: &PropertyValue): Option<u64> {
    match (self) {
        PropertyValue::String(_) => option::none(),
        PropertyValue::Number(number) => option::some(*number),
    }
}
