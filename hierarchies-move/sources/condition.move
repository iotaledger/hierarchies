module hierarchies::statement_condition;

use hierarchies::statement_value::StatementValue;
use std::string::String;

/// StatementValueCondition is a condition that can be applied to a StatementValue.
public enum StatementValueCondition has copy, drop, store {
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    GreaterThan(u64),
    LowerThan(u64),
}

/// Creates a new StatementValueCondition that checks if the value starts with the given text.
public fun new_condition_starts_with(text: String): StatementValueCondition {
    StatementValueCondition::StartsWith(text)
}

/// Creates a new StatementValueCondition that checks if the value ends with the given text.
public fun new_condition_ends_with(text: String): StatementValueCondition {
    StatementValueCondition::EndsWith(text)
}

/// Creates a new StatementValueCondition that checks if the value contains the given text.
public fun new_condition_contains(text: String): StatementValueCondition {
    StatementValueCondition::Contains(text)
}

/// Creates a new StatementValueCondition that checks if the value is greater than the given number.
public fun new_condition_greater_than(value: u64): StatementValueCondition {
    StatementValueCondition::GreaterThan(value)
}

/// Creates a new StatementValueCondition that checks if the value is lower than the given number.
public fun new_condition_lower_than(value: u64): StatementValueCondition {
    StatementValueCondition::LowerThan(value)
}

/// Checks if the condition matches the value.
public fun condition_matches(self: &StatementValueCondition, value: &StatementValue): bool {
    match (self) {
        StatementValueCondition::StartsWith(ref_string) => {
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
        StatementValueCondition::EndsWith(ref_string) => {
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
        StatementValueCondition::Contains(ref_string) => {
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
        StatementValueCondition::GreaterThan(ref_value) => {
            let maybe_value_number = value.as_number();
            if (maybe_value_number.is_none()) {
                return false
            };
            let value_number = maybe_value_number.borrow();
            return *value_number > *ref_value
        },
        StatementValueCondition::LowerThan(ref_value) => {
            let maybe_value_number = value.as_number();
            if (maybe_value_number.is_none()) {
                return false
            };
            let value_number = maybe_value_number.borrow();
            return *value_number < *ref_value
        },
    }
}
