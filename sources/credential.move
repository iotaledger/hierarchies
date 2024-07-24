module htf::credential {

  use sui::vec_map::{VecMap};
  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};


  public fun new_credential(trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>, ctx : &mut TxContext) : Credential {
      Credential {
        id : object::new(ctx),
        issued_by : ctx.sender().to_id(),
        trusted_properties,
      }
  }


  public struct Credential has key, store  {
    id : UID,
    issued_by : ID,
    trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>,
  }


  public(package)  fun issued_by(self : &Credential)  : &ID {
    &self.issued_by
  }

  public(package) fun trusted_properties(self : &Credential) :  &VecMap<TrustedPropertyName, TrustedPropertyValue> {
    &self.trusted_properties
  }
}
