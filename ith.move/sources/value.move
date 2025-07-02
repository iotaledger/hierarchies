module ith::statement_value {
  use std::string::String;

  /// StatementValue can be a String or a Number.
  public enum StatementValue has copy, drop, store {
    String(String),
    Number(u64),
}

  /// Creates a new StatementValue from a String.
  public fun new_statement_value_string(v : String)  : StatementValue {
    StatementValue::String(v)
  }

  /// Creates a new StatementValue from a u64 number.
  public fun new_statement_value_number(v : u64)  : StatementValue {
    StatementValue::Number(v)
  }

  public(package) fun as_string(self : &StatementValue) : Option<String> {
    match (self) {
      StatementValue::String(text) => option::some(*text),
      StatementValue::Number(_) => option::none(),
    }
  }

  public(package) fun as_number(self : &StatementValue) : Option<u64> {
    match (self) {
      StatementValue::String(_) => option::none(),
      StatementValue::Number(number) => option::some(*number),
    }
  }
}
