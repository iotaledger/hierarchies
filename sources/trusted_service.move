
// HTF Notary module
module htf::trusted_service {
  use std::string::String;

  // Trust service is a source of root authorities
  public struct TrustedService has store, key {
    id : UID,
    service_type : String,
    data : vector<TrustedServiceProperty>,
  }

  public struct TrustedServiceProperty  has store {
    name : String,
    value : String,
  }

  public(package) fun new_trust_service(ctx : &mut TxContext, service_type : String) :  TrustedService {
    TrustedService {
      id : object::new(ctx),
      service_type,
      data : vector[] ,
    }
  }

}
