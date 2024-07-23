
module htf::trusted_property {
  use std::string::String;

  public struct TrustedPropertyName  has copy, drop, store {
    // initially its a string, but it could be more complex structure that implements copy and drop
    name : String,
  }

  public struct TrustedPropertyValue has copy, drop, store {
    // initially its a string, but it could be more complex structure that implements copy and drop
     value : String
  }


  public fun new_trusted_property_value(v : String)  : TrustedPropertyValue {
    TrustedPropertyValue {
      value : v
    }

  }

  public fun new_trusted_property_name(v : String) : TrustedPropertyName {
    TrustedPropertyName {
      name: v,
    }
  }



}
