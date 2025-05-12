module ith::statement_condition {
  use std::string::String;
  use ith::statement_value::{StatementValue};

  public struct StatementValueCondition has store, copy, drop {
    starts_with : Option<String>,
    ends_with : Option<String>,
    contains : Option<String>,
    greater_than : Option<u64>,
    lower_than : Option<u64>,
  }

  public fun new_statement_value_condition(starts_with : Option<String>, ends_with : Option<String>, contains : Option<String>, greater_than : Option<u64>, lower_than : Option<u64>) : StatementValueCondition {
    StatementValueCondition {
      starts_with,
      ends_with,
      contains,
      greater_than,
      lower_than,
    }
  }

  public fun set_starts_with(self: &mut StatementValueCondition, value: Option<String>) {
    self.starts_with = value;
  }

  public fun set_ends_with(self: &mut StatementValueCondition, value: Option<String>) {
    self.ends_with = value;
  }

  public fun set_contains(self: &mut StatementValueCondition, value: Option<String>) {
    self.contains = value;
  }

  public fun set_greater_than(self: &mut StatementValueCondition, value: Option<u64>) {
    self.greater_than = value;
  }

  public fun set_lower_than(self: &mut StatementValueCondition, value: Option<u64>) {
    self.lower_than = value;
  }


  public fun as_starts_with(self : &StatementValueCondition) : Option<String> {
    self.starts_with
  }

  public fun as_ends_with(self : &StatementValueCondition) : Option<String> {
    self.ends_with
  }

  public fun as_lower_than(self : &StatementValueCondition) : Option<u64> {
    self.lower_than
  }

  public fun as_greater_than(self : &StatementValueCondition) : Option<u64> {
    self.greater_than
  }

  public fun as_contains(self : &StatementValueCondition) : Option<String> {
    self.contains
  }

  public fun is_starts_with(self : &StatementValueCondition) : bool {
    self.starts_with.is_some()
  }

  public fun is_ends_with(self : &StatementValueCondition) : bool {
    self.ends_with.is_some()
  }

  public fun is_contains(self : &StatementValueCondition) : bool {
    self.contains.is_some()
  }

  public fun is_greater_than(self : &StatementValueCondition) : bool {
    self.greater_than.is_some()
  }

  public fun is_lower_than(self : &StatementValueCondition) : bool {
    self.lower_than.is_some()
  }

  public fun matches_condition(self : &StatementValueCondition,  value : &StatementValue) : bool {
    if (self.is_starts_with()) {
      let mut maybe_value_string = value.as_string();
      if (maybe_value_string.is_none()) {
        return false
      };
      let value_string = maybe_value_string.extract();
      if (value_string.length() < self.as_starts_with().borrow().length()) {
        return false
      };
      return value_string.index_of(self.as_starts_with().borrow()) == 0
    };

    if (self.is_ends_with()) {
      let mut maybe_value_string = value.as_string();
      if (maybe_value_string.is_none()) {
        return false
      };
      let value_string = maybe_value_string.extract();
      if (value_string.length() < self.as_ends_with().borrow().length()) {
        return false
      };
      return value_string.index_of(self.as_ends_with().borrow()) == value_string.length() - self.as_ends_with().borrow().length()
    };

    if (self.is_contains()) {
      let mut maybe_value_string = value.as_string();
      if (maybe_value_string.is_none()) {
        return false
      };
      let value_string = maybe_value_string.extract();
      if (value_string.length() < self.as_contains().borrow().length()) {
        return false
      };
      let value_string_len = value_string.length();
      let index = value_string.index_of(self.as_contains().borrow());
      if (index == value_string_len) {
        return false
      };
    };

    if (self.is_greater_than()) {
      let mut maybe_value_number = value.as_number();
      if (maybe_value_number.is_none()) {
        return false
      };
      let value_number = maybe_value_number.extract();
      return value_number > *self.as_greater_than().borrow()
    };

    if (self.is_lower_than()) {
      let mut maybe_value_number = value.as_number();
      if (maybe_value_number.is_none()) {
        return false
      };
      let value_number = maybe_value_number.extract();
      return value_number < *self.as_lower_than().borrow()
    };

    false
  }
}
