module htf::trusted_constraint {

  use sui::object::{Self, UID};
  use sui::vec_map::{Self, VecMap};
  use sui::vec_set::{Self, VecSet};

  use htf::trusted_property::{TrustedPropertyValue, TrustedPropertyName};


  public(package) fun new_trusted_property_constraints() : TrustedPropertyConstraints {
    TrustedPropertyConstraints {
      data  : vec_map::empty(),
    }
  }

  public struct TrustedPropertyConstraints has store {
    data : VecMap<TrustedPropertyName, TrustedPropertyConstraint>
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
      // no name
      return false
    };
    self.data.get(property_name).allowed_values.contains(value)
  }

  public(package) fun add_constraint(self : &mut TrustedPropertyConstraints, name : TrustedPropertyName, constraint : TrustedPropertyConstraint)  {
    self.data.insert(name, constraint)
  }

  public(package) fun new_trusted_property_constraint(property_name : TrustedPropertyName, allowed_values : VecSet<TrustedPropertyValue>) : TrustedPropertyConstraint {
    TrustedPropertyConstraint {
      property_name,
      allowed_values,
      allow_any: false,
    }
  }

  public struct TrustedPropertyConstraint has  store, copy, drop {
    property_name : TrustedPropertyName,
    allowed_values : VecSet<TrustedPropertyValue>,
    allow_any : bool,
  }

  public(package) fun allowed_values(self : &TrustedPropertyConstraint) : &VecSet<TrustedPropertyValue> {
    &self.allowed_values
  }

  public(package) fun property_name(self : &TrustedPropertyConstraint) : &TrustedPropertyName {
    &self.property_name
  }
}
