
module ith::trusted_property {
  use std::string::String;


  public struct TrustedPropertyName  has copy, drop, store {
    names : vector<String>,
  }


  public fun names(self : &TrustedPropertyName) : &vector<String> {
    &self.names
  }


  public struct TrustedPropertyValue has copy, drop, store {
    text : Option<String>,
    number : Option<u64>,
  }

  public fun as_string(self : &TrustedPropertyValue) : Option<String> {
    self.text
  }

  public fun as_number(self : &TrustedPropertyValue) : Option<u64> {
    self.number
  }

  public fun is_string(self : &TrustedPropertyValue) : bool {
    self.text.is_some()
  }

  public fun is_number(self : &TrustedPropertyValue) : bool {
    self.number.is_some()
  }

  // length return optional value as the Value in the future could be a number or more complex structure
  public fun length(self : &TrustedPropertyValue) :  Option<u64> {
    if (self.is_string()) {
      let text = self.as_string();
      option::some(text.borrow().length())
    } else {
      option::none()
    }
  }

  public fun new_property_value_string(v : String)  : TrustedPropertyValue {
        TrustedPropertyValue{
          text: option::some(v),
          number : option::none(),
        }
  }

  public fun new_property_value_number(v : u64)  : TrustedPropertyValue {
    TrustedPropertyValue {
      text: option::none(),
      number: option::some(v),
    }
  }


  public fun new_property_name(v : String) : TrustedPropertyName {
    let mut names = vector::empty();
    names.push_back(v);
    TrustedPropertyName {
      names,
    }
  }

  public fun new_property_name_from_vector(v : vector<String>) : TrustedPropertyName {
    TrustedPropertyName {
      names : v,
    }
  }
}
