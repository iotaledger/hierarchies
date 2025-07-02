module ith::accreditation;

use iota::vec_map::VecMap;
use ith::{
    statement::{Self, Statement},
    statement_name::StatementName,
    statement_value::StatementValue,
    utils
};
use std::string::String;

public struct Accreditations has store {
    statements: vector<Accreditation>,
}

  /// Creates a new empty list of Accreditations.
  public fun new_empty_accreditations() : Accreditations {
    Accreditations {
        statements: vector::empty(),
    }
}

  /// Creates a collection of Accreditations.
  public fun new_accreditations(statements : vector<Accreditation>) : Accreditations {
    Accreditations {
        statements: statements,
    }
}

  /// Adds an accredited Statement to the list of accreditations.
  public(package) fun add_accreditation(self : &mut Accreditations, accredited_statement : Accreditation) {
    self.statements.push_back(accredited_statement);
}

/// Check if the statements are allowed by any of the accredited statements.
/// The statements are allowed if the accredited statement is not expired and the statement value matches the condition.
public(package) fun are_statements_allowed(
    self: &Accreditations,
    statements: &VecMap<StatementName, StatementValue>,
    current_time_ms: u64,
): bool {
    let statement_names = statements.keys();
    let mut idx_statement_names = 0;

    while (idx_statement_names < statement_names.length()) {
        let statement_name = statement_names[idx_statement_names];
        let statement_value = statements.get(&statement_name);

        if (!self.is_statement_allowed(&statement_name, statement_value, current_time_ms)) {
            return false
        };
        idx_statement_names = idx_statement_names + 1;
    };
    return true
}

/// Check if the statement is allowed by any of the accredited statements.
public(package) fun is_statement_allowed(
    self: &Accreditations,
    statement_name: &StatementName,
    statement_value: &StatementValue,
    current_time_ms: u64,
): bool {
    let len_statements_to_attest = self.statements.length();
    let mut idx_statements_to_attest = 0;

    while (idx_statements_to_attest < len_statements_to_attest) {
        let accreditation = &self.statements[idx_statements_to_attest];
        let maybe_statement = accreditation.statements.try_get(statement_name);

        if (maybe_statement.is_none()) {
            continue
        };
        if (
            maybe_statement
                .borrow()
                .matches_name_value(statement_name, statement_value, current_time_ms)
        ) {
            return true
        };
        idx_statements_to_attest = idx_statements_to_attest + 1;
    };
    return false
}

/// Check the compliance of the statements. The compliance is met if all set of statements names and values is at most the set of accredited statements.
public(package) fun are_statements_compliant(
    self: &Accreditations,
    statements: &vector<Statement>,
    current_time_ms: u64,
): bool {
    let mut idx = 0;
    while (idx < statements.length()) {
        let statement = statements[idx];
        if (!self.is_statement_compliant(&statement, current_time_ms)) {
            return false
        };
        idx = idx + 1;
    };
    return true
}

/// Check the compliance of the statement. The compliance is met if all set of statement names and values is at most the set of accredited statements.
public(package) fun is_statement_compliant(
    self: &Accreditations,
    statement: &Statement,
    current_time_ms: u64,
): bool {
    let len_statements = self.statements.length();
    let mut idx_statements = 0;
    let mut want_statements: vector<StatementValue> = utils::copy_vector(statement
        .allowed_values()
        .keys());

    while (idx_statements < len_statements) {
        let accredited_statement = &self.statements[idx_statements];

        let value_condition = accredited_statement.statements.try_get(statement.statement_name());
        if (value_condition.is_none()) {
            continue
        };

        let mut len_want_statements = want_statements.length();
        let mut idx_want_statements = 0;
        while (idx_want_statements < len_want_statements) {
            let statement_value = want_statements[idx_want_statements];
            if (value_condition.borrow().matches_value(&statement_value, current_time_ms)) {
                want_statements.remove(idx_want_statements);
                len_want_statements = len_want_statements - 1;
            };
            idx_want_statements = idx_want_statements + 1;
        };
        idx_statements = idx_statements + 1;
    };

    // All wanted statements have been found
    if (want_statements.length() == 0) {
        return true
    };
    return false
}

public(package) fun add_accredited_statement(self: &mut Accreditations, permission: Accreditation) {
    self.statements.push_back(permission);
}

public(package) fun accredited_statements(self: &Accreditations): &vector<Accreditation> {
    &self.statements
}

public(package) fun remove_accredited_statement(self: &mut Accreditations, id: &ID) {
    let mut idx = self.find_accredited_statement_id(id);
    if (idx.is_none()) {
        return
    };
    let Accreditation {
        id,
        statements: _,
        accredited_by: _,
    } = self.statements.remove(idx.extract());
    object::delete(id);
}

public(package) fun find_accredited_statement_id(self: &Accreditations, id: &ID): Option<u64> {
    let mut idx = 0;
    while (idx < self.statements.length()) {
        if (self.statements[idx].id.to_inner() == *id) {
            return option::some(idx)
        };
        idx = idx + 1;
    };
    option::none()
}

/// Accreditation represents statements that are accredited by a third party.
public struct Accreditation has key, store {
    id: UID,
    accredited_by: String,
    statements: VecMap<StatementName, Statement>,
}

public fun new_accreditation(statements: vector<Statement>, ctx: &mut TxContext): Accreditation {
    let statements_map = statement::to_map_of_statements(statements);

    Accreditation {
        id: object::new(ctx),
        accredited_by: ctx.sender().to_string(),
        statements: statements_map,
    }
}

public(package) fun id(self: &Accreditation): &UID {
    &self.id
}

public(package) fun accredited_by(self: &Accreditation): &String {
    &self.accredited_by
}

public(package) fun statements(self: &Accreditation): &VecMap<StatementName, Statement> {
    &self.statements
}

// ===== Test-only Functions =====

#[test_only]
public(package) fun destroy_accreditation(self: Accreditation) {
    let Accreditation {
        id: id,
        accredited_by: _,
        statements: _,
    } = self;

    object::delete(id);
}

#[test_only]
public(package) fun destroy_accreditations(self: Accreditations) {
    let Accreditations { statements } = self;

    statements.destroy!(|accreditation| {
        destroy_accreditation(accreditation);
    });
}
