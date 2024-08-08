module htf::trusted_constraint {

  use iota::object::{Self, UID};
  use iota::vec_map::{Self, VecMap};
  use iota::vec_set::{Self, VecSet};

  use htf::trusted_property::{TrustedPropertyValue, TrustedPropertyName};
  use htf::utils;


  public(package) fun new_trusted_property_constraints() : TrustedPropertyConstraints {
    TrustedPropertyConstraints {
      data  : vec_map::empty(),
    }
  }

  public struct TrustedPropertyConstraints has store {
    data : VecMap<TrustedPropertyName, TrustedPropertyConstraint>
  }

  public(package)  fun data(self : &TrustedPropertyConstraints) : &VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &self.data
  }

  public(package)  fun data_mut(self : &mut TrustedPropertyConstraints) : &mut VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &mut self.data
  }


  public(package) fun are_properties_correct(self : &TrustedPropertyConstraints, properties : &VecMap<TrustedPropertyName, TrustedPropertyValue>)  : bool {
      let property_names = properties.keys() ;
      let mut  idx = 0;

      while (idx < property_names.length())  {
        if (! self.is_property_correct(&property_names[idx], properties.get(&property_names[idx]))) {
          return false
        };
        idx = idx +1;
      };

      true
  }

  public(package) fun is_property_correct(self : &TrustedPropertyConstraints, property_name : &TrustedPropertyName, value : &TrustedPropertyValue) : bool {
    if ( ! self.data.contains(property_name) ) {
      return false
    };
    self.data.get(property_name).matches_property(property_name, value)
  }

  public(package) fun add_constraint(self : &mut TrustedPropertyConstraints, name : TrustedPropertyName, constraint : TrustedPropertyConstraint)  {
    self.data.insert(name, constraint)
  }

  public(package) fun new_trusted_property_constraint(property_name : TrustedPropertyName, allowed_values : VecSet<TrustedPropertyValue>, allow_any : bool) : TrustedPropertyConstraint {
    TrustedPropertyConstraint {
      property_name,
      allowed_values,
      allow_any,
    }
  }

  public struct TrustedPropertyConstraint has  store, copy, drop {
    property_name : TrustedPropertyName,
    // allow only set of values
    allowed_values : VecSet<TrustedPropertyValue>,
    // allow_any - takes a precedence over the allowed_values
    allow_any : bool,
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

   utils::contains_all_from(self.allowed_values.keys(), constraint.allowed_values.keys())
  }

  public(package) fun matches_property(self: &TrustedPropertyConstraint, name: &TrustedPropertyName, value: &TrustedPropertyValue) : bool {
    self.matches_name(name) && self.matches_value(value)
  }

  public(package) fun matches_name(self : &TrustedPropertyConstraint, name : &TrustedPropertyName) : bool {
      self.property_name == name
  }

  public(package) fun matches_value(self : &TrustedPropertyConstraint, value : &TrustedPropertyValue) : bool {
    if ( self.allow_any ) {
      return true
    };
    self.allowed_values.contains(value)
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
}
