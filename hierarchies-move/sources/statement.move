module hierarchies::statement;

use hierarchies::{
    statement_condition::StatementValueCondition,
    statement_name::StatementName,
    statement_value::StatementValue
};
use iota::{vec_map::{Self, VecMap}, vec_set::VecSet};

// Statements is a struct that contains a map of StatementName to Statement
public struct Statements has store {
    data: VecMap<StatementName, Statement>,
}

// The evaluation order: allow_any => condition => allowed_values
// The evaluation order is determined by the possible size of the set of values
// that match the condition.
public struct Statement has copy, drop, store {
    statement_name: StatementName,
    // allow only values that are in the set
    allowed_values: VecSet<StatementValue>,
    // Allow only values that match the condition.
    condition: Option<StatementValueCondition>,
    // If true, the statement is not applied, any value is allowed
    allow_any: bool,
    // The time span of the statement
    timespan: Timespan,
}

/// Creates a new Statement
public fun new_statement(
    statement_name: StatementName,
    allowed_values: VecSet<StatementValue>,
    allow_any: bool,
    condition: Option<StatementValueCondition>,
): Statement {
    Statement {
        statement_name,
        allowed_values,
        condition,
        allow_any,
        timespan: new_empty_timespan(),
    }
}

public(package) fun new_statements(): Statements {
    Statements {
        data: vec_map::empty(),
    }
}

public(package) fun data(self: &Statements): &VecMap<StatementName, Statement> {
    &self.data
}

public(package) fun data_mut(self: &mut Statements): &mut VecMap<StatementName, Statement> {
    &mut self.data
}

public(package) fun add_statement(self: &mut Statements, statement: Statement) {
    let name = statement.statement_name;
    self.data.insert(name, statement)
}

public(package) fun allowed_values(self: &Statement): &VecSet<StatementValue> {
    &self.allowed_values
}

public(package) fun statement_name(self: &Statement): &StatementName {
    &self.statement_name
}

public(package) fun matches_name_value(
    self: &Statement,
    name: &StatementName,
    value: &StatementValue,
    current_time_ms: u64,
): bool {
    self.matches_name(name) && self.matches_value(value, current_time_ms)
}

public(package) fun matches_name(self: &Statement, name: &StatementName): bool {
    // considering the statement name is a.b.c
    // the allowed name should be equal a.b.c or longer
    let len_statement = self.statement_name.names().length();
    let len_names = name.names().length();

    // if it's longer than the name, it's not possible to match
    if (len_statement > len_names) {
        return false
    };

    let mut idx = 0;
    while (idx < len_statement) {
        if (self.statement_name.names()[idx] != name.names()[idx]) {
            // if you have a.b.c and a.b.d, it is not possible to match
            return false
        };
        idx = idx + 1;
    };

    true
}

public(package) fun matches_value(
    self: &Statement,
    value: &StatementValue,
    current_time_ms: u64,
): bool {
    if (!self.timespan.timestamp_matches(current_time_ms)) {
        return false
    };

    if (self.allow_any) {
        return true
    };
    if (self.condition.is_some()) {
        if (self.condition.borrow().condition_matches(value)) {
            return true
        }
    };
    self.allowed_values.contains(value)
}

public(package) fun revoke(self: &mut Statement, valid_to_ms: u64) {
    self.timespan.valid_until_ms = option::some(valid_to_ms)
}

public(package) fun to_map_of_statements(
    statements: vector<Statement>,
): VecMap<StatementName, Statement> {
    let mut idx = 0;
    let mut map: VecMap<StatementName, Statement> = vec_map::empty();
    while (idx < statements.length()) {
        let statement = statements[idx];
        map.insert(*statement.statement_name(), statement);
        idx = idx + 1;
    };
    return map
}

// ===== Timespan ========

/// Represents a time statement. The valid_from_ms and valid_until_ms are
/// optional, if they are not set, the statement is valid for all time.
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
    if (self.valid_until_ms.is_some() && *self.valid_until_ms.borrow() < now_ms) {
        return false
    };
    true
}

// ===== Test-only Functions =====
#[test_only]
public(package) fun destroy_statements(statements: Statements) {
    let Statements { data: _ } = statements;
}
