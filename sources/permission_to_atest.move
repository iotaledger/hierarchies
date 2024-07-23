// HTF Notary module
module htf::permission_to_atest {

  use std::string::String;
  use sui::vec_map::{Self, VecMap};

  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use htf::trusted_constraint::{TrustedPropertyConstraint};

  public struct PersmissionsToAtest has store {
    permissions_to_atest : vector<PermissionToAtest>,
  }

  /// PermissionToAtest can be created only by the HTF module
  public struct PermissionToAtest has store, key {
    id : UID,
    federation_id : String,
    created_by : String,
    trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
  }

  public(package) fun new_permission_to_atest(federation_id : String, constraints : VecMap<TrustedPropertyName, TrustedPropertyConstraint>, ctx : &mut TxContext) : PermissionToAtest {
    PermissionToAtest {
        id : object::new(ctx),
        federation_id,
        created_by : ctx.sender().to_string(),
        trusted_properties: constraints,
    }
   }

  public(package) fun new_permissions_to_atest() : PersmissionsToAtest {
    PersmissionsToAtest {
      permissions_to_atest: vector::empty(),
    }
  }

  public(package)  fun federation_id(self : &PermissionToAtest) : &String {
    &self.federation_id
  }

  public(package) fun created_by(self : &PermissionToAtest) : &String {
    &self.created_by
  }

  public(package) fun trusted_properties(self : &PermissionToAtest) : &VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &self.trusted_properties
  }

  public(package) fun add_permission_to_atest(self : &mut PersmissionsToAtest, permission_to_atest : PermissionToAtest) {
    self.permissions_to_atest.push_back(permission_to_atest);
  }

  /// checks if all constraints matches the given in Accredidations
  public(package) fun are_values_permitted(self : &PersmissionsToAtest, trusted_properties: &VecMap<TrustedPropertyName, TrustedPropertyValue> ) :bool {
    let property_names = trusted_properties.keys() ;
    let mut idx_property_names = 0;

    while ( idx_property_names < property_names.length() ) {
      let property_name = property_names[idx_property_names];
      let property_value = trusted_properties.get(&property_name);

      if (! self.is_value_permitted(&property_name, property_value)  ) {
        return false
      };
      idx_property_names = idx_property_names + 1;
    };

    return true
  }


  public(package) fun is_value_permitted(self : &PersmissionsToAtest, property_name : &TrustedPropertyName, property_value : &TrustedPropertyValue) :  bool {
    let len_permissions_to_atest = self.permissions_to_atest.length();
    let mut idx_permissions_to_atest = 0;

    while (idx_permissions_to_atest < len_permissions_to_atest) {
      let acreditation = &self.permissions_to_atest[idx_permissions_to_atest];

      let maybe_property_constraint = acreditation.trusted_properties.try_get(property_name) ;
      if ( maybe_property_constraint.is_none()) {
        continue
      };
      if (maybe_property_constraint.borrow().matches_property(property_name, property_value)) {
        return true
      };
      idx_permissions_to_atest = idx_permissions_to_atest + 1;
    };


    return false
  }
}
