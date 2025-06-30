
module ith::statement_value {
  use std::string::String;

  // Is the enum actually supported
  // Use the enum
  // How can we determine the type of the value. Deserialization
  public enum StatementValueEnum has copy, drop, store {
    String(String),
    Number(u64),
  }


  public struct StatementValue has copy, drop, store {
    text : Option<String>,
    number : Option<u64>,
  }

  public fun as_string(self : &StatementValue) : Option<String> {
    self.text
  }

  public fun as_number(self : &StatementValue) : Option<u64> {
    self.number
  }

  public fun is_string(self : &StatementValue) : bool {
    self.text.is_some()
  }

  public fun is_number(self : &StatementValue) : bool {
    self.number.is_some()
  }

  // length return optional value as the Value in the future could be a number or more complex structure
  public fun length(self : &StatementValue) :  Option<u64> {
    if (self.is_string()) {
      let text = self.as_string();
      option::some(text.borrow().length())
    } else {
      option::none()
    }
  }

  public fun new_property_value_string(v : String)  : StatementValue {
        StatementValue{
          text: option::some(v),
          number : option::none(),
        }
  }

  public fun new_property_value_number(v : u64)  : StatementValue {
    StatementValue {
      text: option::none(),
      number: option::some(v),
    }
  }
}
