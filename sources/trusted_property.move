
module htf::trusted_property {
  use std::string::String;
  use std::option::{Self, Option};
  use std::type_name::{Self, TypeName};

  public struct TrustedPropertyName  has copy, drop, store {
    // initially its a string, but it could be more complex structure that implements copy and drop
    name : String,
  }

  public enum TrustedPropertyValue has copy, drop, store {
    String(String),
    Number(u64),
  }


  // length return optional value as the Value in the future could be a number or more complex structure
  public fun length(self : &TrustedPropertyValue) :  Option<u64> {
    match (self) {
      TrustedPropertyValue::String(v) => option::some(v.length()),
      TrustedPropertyValue::Number(_) => option::none(),
    };
    option::none()
  }

  public fun as_string(self : &TrustedPropertyValue) : Option<String> {
    match (self) {
      TrustedPropertyValue::String(v) => option::some(*v),
      TrustedPropertyValue::Number(_) => option::none(),
    };
    option::none()
  }

  public fun as_number(self : &TrustedPropertyValue) : Option<u64> {
    match (self) {
      TrustedPropertyValue::Number(v) => option::some(*v),
      TrustedPropertyValue::String(_) => option::none(),
    };
    option::none()
  }


  public fun new_property_value_string(v : String)  : TrustedPropertyValue {
    TrustedPropertyValue::String(v)
  }

  public fun new_property_value_number(v : u64)  : TrustedPropertyValue {
    TrustedPropertyValue::Number(v)
  }


  public fun new_property_name(v : String) : TrustedPropertyName {
    TrustedPropertyName {
      name: v,
    }
  }
}
