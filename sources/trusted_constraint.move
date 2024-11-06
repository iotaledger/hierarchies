module ith::trusted_constraint {

  use iota::vec_map::{Self, VecMap};
  use iota::vec_set::VecSet;
  use std::string::String;

  use ith::trusted_property::{TrustedPropertyValue, TrustedPropertyName};
  use ith::utils;

  public struct TrustedPropertyConstraints has store {
    data : VecMap<TrustedPropertyName, TrustedPropertyConstraint>
  }

  // The evaluation order: allow_any => expression => allowed_values
  public struct TrustedPropertyConstraint has  store, copy, drop {
    property_name : TrustedPropertyName,
    // allow only set of values
    allowed_values : VecSet<TrustedPropertyValue>,
    // allow only values that match the expression.
    expression : Option<TrustedPropertyExpression>,
    // allow_any - takes a precedence over the allowed_values
    allow_any : bool,

    timespan : Timespan,
  }

  public fun new_trusted_property_constraint(property_name : TrustedPropertyName, allowed_values : VecSet<TrustedPropertyValue>, allow_any : bool, expression : Option<TrustedPropertyExpression>) : TrustedPropertyConstraint {
    TrustedPropertyConstraint {
      property_name,
      allowed_values,
      expression,
      allow_any,
      timespan: new_empty_timespan(),
    }
  }

  public struct Timespan has store, copy, drop {
    valid_from_ms : Option<u64>,
    valid_until_ms : Option<u64>,
  }


  public(package) fun new_timespan(valid_from_ms : Option<u64>, valid_until_ms : Option<u64>) : Timespan {
    Timespan {
      valid_from_ms,
      valid_until_ms,
    }
  }

  public(package) fun new_empty_timespan() : Timespan {
    Timespan {
      valid_from_ms: option::none(),
      valid_until_ms: option::none(),
    }
  }

  public(package) fun matches_time(self : &Timespan, now_ms : u64) : bool {
    if (self.valid_from_ms.is_some() && *self.valid_from_ms.borrow() > now_ms) {
      return false
    };
    if (self.valid_until_ms.is_some() && *self.valid_until_ms.borrow() < now_ms) {
      return false
    };
    true
  }


  public struct TrustedPropertyExpression has store, copy, drop {
    starts_with : Option<String>,
    ends_with : Option<String>,
    contains : Option<String>,
    greater_than : Option<u64>,
    lower_than : Option<u64>,
  }

  public fun new_trusted_property_expression(starts_with : Option<String>, ends_with : Option<String>, contains : Option<String>, greater_than : Option<u64>, lower_than : Option<u64>) : TrustedPropertyExpression {
    TrustedPropertyExpression {
      starts_with,
      ends_with,
      contains,
      greater_than,
      lower_than,
    }
  }

  public fun set_starts_with(self: &mut TrustedPropertyExpression, value: Option<String>) {
    self.starts_with = value;
  }

  public fun set_ends_with(self: &mut TrustedPropertyExpression, value: Option<String>) {
    self.ends_with = value;
  }

  public fun set_contains(self: &mut TrustedPropertyExpression, value: Option<String>) {
    self.contains = value;
  }

  public fun set_greater_than(self: &mut TrustedPropertyExpression, value: Option<u64>) {
    self.greater_than = value;
  }

  public fun set_lower_than(self: &mut TrustedPropertyExpression, value: Option<u64>) {
    self.lower_than = value;
  }


  public fun as_starts_with(self : &TrustedPropertyExpression) : Option<String> {
    self.starts_with
  }

  public fun as_ends_with(self : &TrustedPropertyExpression) : Option<String> {
    self.ends_with
  }

  public fun as_lower_than(self : &TrustedPropertyExpression) : Option<u64> {
    self.lower_than
  }

  public fun as_greater_than(self : &TrustedPropertyExpression) : Option<u64> {
    self.greater_than
  }

  public fun as_contains(self : &TrustedPropertyExpression) : Option<String> {
    self.contains
  }

  public fun is_starts_with(self : &TrustedPropertyExpression) : bool {
    self.starts_with.is_some()
  }

  public fun is_ends_with(self : &TrustedPropertyExpression) : bool {
    self.ends_with.is_some()
  }

  public fun is_contains(self : &TrustedPropertyExpression) : bool {
    self.contains.is_some()
  }

  public fun is_greater_than(self : &TrustedPropertyExpression) : bool {
    self.greater_than.is_some()
  }

  public fun is_lower_than(self : &TrustedPropertyExpression) : bool {
    self.lower_than.is_some()
  }

  public(package) fun new_trusted_property_constraints() : TrustedPropertyConstraints {
    TrustedPropertyConstraints {
      data  : vec_map::empty(),
    }
  }


  public(package)  fun data(self : &TrustedPropertyConstraints) : &VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &self.data
  }

  public(package)  fun data_mut(self : &mut TrustedPropertyConstraints) : &mut VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &mut self.data
  }


  public(package) fun are_properties_correct(self : &TrustedPropertyConstraints, properties : &VecMap<TrustedPropertyName, TrustedPropertyValue>, current_time_ms : u64)  : bool {
      let property_names = properties.keys() ;
      let mut  idx = 0;

      while (idx < property_names.length())  {
        if (! self.is_property_correct(&property_names[idx], properties.get(&property_names[idx]), current_time_ms)) {
          return false
        };
        idx = idx +1;
      };

      true
  }

  public(package) fun is_property_correct(self : &TrustedPropertyConstraints, property_name : &TrustedPropertyName, value : &TrustedPropertyValue, current_time_ms : u64) : bool {
    if ( ! self.data.contains(property_name) ) {
      return false
    };
    self.data.get(property_name).matches_property(property_name, value, current_time_ms)
  }

  public(package) fun add_constraint(self : &mut TrustedPropertyConstraints, name : TrustedPropertyName, constraint : TrustedPropertyConstraint)  {
    self.data.insert(name, constraint)
  }


  public(package) fun allowed_values(self : &TrustedPropertyConstraint) : &VecSet<TrustedPropertyValue> {
    &self.allowed_values
  }

  public(package) fun property_name(self : &TrustedPropertyConstraint) : &TrustedPropertyName {
    &self.property_name
  }

  public(package) fun matches_contraint(self : &TrustedPropertyConstraint, constraint : &TrustedPropertyConstraint)  : bool {
    if (constraint.allow_any) {
      return self.allow_any
    };
    if (constraint.expression.is_some()) {
      if (self.expression.is_some()) {
        return self.expression == constraint.expression
      }
    };
   utils::contains_all_from(self.allowed_values.keys(), constraint.allowed_values.keys())
  }

  public(package) fun matches_property(self: &TrustedPropertyConstraint, name: &TrustedPropertyName, value: &TrustedPropertyValue, current_time_ms : u64) : bool {
    self.matches_name(name) && self.matches_value(value, current_time_ms)
  }

  public(package) fun matches_name(self : &TrustedPropertyConstraint, name : &TrustedPropertyName) : bool {
      // considering the constraint name is a.b.c
      // the allowed name should be equal a.b.c or longer
      let len_constraint = self.property_name.names().length();
      let len_names = name.names().length();

      // if contraint is longer than the name, it is not possible to match
      if (len_constraint > len_names) {
        return false
      };

      let mut idx = 0;
      while (idx < len_constraint) {
        if (self.property_name.names()[idx] != name.names()[idx]) {
          // if you have a.b.c and a.b.d, it is not possible to match
          return false
        };
        idx = idx + 1;
      };

      true
  }

  public(package) fun matches_value(self : &TrustedPropertyConstraint, value : &TrustedPropertyValue, current_time_ms : u64) : bool {
    if ( ! self.timespan.matches_time(current_time_ms) ) {
      return false
    };

    if ( self.allow_any ) {
      return true
    };
    if (self.expression.is_some()) {
      if (Self::matches_expression(self.expression.borrow(), value)) {
        return true
      }
    };
    self.allowed_values.contains(value)
  }

  public(package) fun revoke_constraint(self : &mut TrustedPropertyConstraint, valid_to_ms : u64) {
    self.timespan.valid_until_ms = option::some(valid_to_ms)
  }


  public(package) fun to_map_of_constraints(constraints : vector<TrustedPropertyConstraint>) : VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    let mut idx = 0;
    let mut map : VecMap<TrustedPropertyName, TrustedPropertyConstraint> = vec_map::empty();
    while ( idx < constraints.length() ) {
      let constraint = constraints[idx];
      map.insert(*constraint.property_name(), constraint);
      idx = idx + 1;

    };
    return map
  }

  public(package) fun matches_expression(exp : &TrustedPropertyExpression,  value : &TrustedPropertyValue) : bool {
    if (exp.is_starts_with()) {
      let mut maybe_value_string = value.as_string();
      if (maybe_value_string.is_none()) {
        return false
      };
      let value_string = maybe_value_string.extract();
      if (value_string.length() < exp.as_starts_with().borrow().length()) {
        return false
      };
      return value_string.index_of(exp.as_starts_with().borrow()) == 0
    };

    if (exp.is_ends_with()) {
      let mut maybe_value_string = value.as_string();
      if (maybe_value_string.is_none()) {
        return false
      };
      let value_string = maybe_value_string.extract();
      if (value_string.length() < exp.as_ends_with().borrow().length()) {
        return false
      };
      return value_string.index_of(exp.as_ends_with().borrow()) == value_string.length() - exp.as_ends_with().borrow().length()
    };

    if (exp.is_contains()) {
      let mut maybe_value_string = value.as_string();
      if (maybe_value_string.is_none()) {
        return false
      };
      let value_string = maybe_value_string.extract();
      if (value_string.length() < exp.as_contains().borrow().length()) {
        return false
      };
      let value_string_len = value_string.length();
      let index = value_string.index_of(exp.as_contains().borrow());
      if (index == value_string_len) {
        return false
      };
    };

    if (exp.is_greater_than()) {
      let mut maybe_value_number = value.as_number();
      if (maybe_value_number.is_none()) {
        return false
      };
      let value_number = maybe_value_number.extract();
      return value_number > *exp.as_greater_than().borrow()
    };

    if (exp.is_lower_than()) {
      let mut maybe_value_number = value.as_number();
      if (maybe_value_number.is_none()) {
        return false
      };
      let value_number = maybe_value_number.extract();
      return value_number < *exp.as_lower_than().borrow()
    };

    false
  }
}



