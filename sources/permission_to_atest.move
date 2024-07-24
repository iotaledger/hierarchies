// HTF Notary module
module htf::permission_to_attest {

  use std::string::String;
  use sui::vec_map::{Self, VecMap};

  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use htf::trusted_constraint::{TrustedPropertyConstraint};

  public struct PersmissionsToAttest has store {
    permissions_to_attest : vector<PermissionToAttest>,
  }

  /// PermissionToAttest can be created only by the HTF module
  public struct PermissionToAttest has store, key {
    id : UID,
    federation_id : ID,
    created_by : String,
    trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
  }

  public(package) fun new_permission_to_attest(federation_id: ID, constraints: VecMap<TrustedPropertyName, TrustedPropertyConstraint>, ctx : &mut TxContext) : PermissionToAttest {
    PermissionToAttest {
        id : object::new(ctx),
        federation_id,
        created_by : ctx.sender().to_string(),
        trusted_properties: constraints,
    }
   }

  public(package) fun new_permissions_to_attest(): PersmissionsToAttest {
    PersmissionsToAttest {
      permissions_to_attest: vector::empty(),
    }
  }

  public(package)  fun federation_id(self : &PermissionToAttest) : &ID {
    &self.federation_id
  }

  public(package) fun created_by(self : &PermissionToAttest) : &String {
    &self.created_by
  }

  public(package) fun trusted_properties(self : &PermissionToAttest) : &VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &self.trusted_properties
  }

  public(package) fun add_permission_to_attest(self : &mut PersmissionsToAttest, permission_to_attest : PermissionToAttest) {
    self.permissions_to_attest.push_back(permission_to_attest);
  }

  /// checks if all constraints matches the given in Accredidations
  public(package) fun are_values_permitted(self : &PersmissionsToAttest, trusted_properties: &VecMap<TrustedPropertyName, TrustedPropertyValue> ) :bool {
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


  public(package) fun is_value_permitted(self : &PersmissionsToAttest, property_name : &TrustedPropertyName, property_value : &TrustedPropertyValue) :  bool {
    let len_permissions_to_attest = self.permissions_to_attest.length();
    let mut idx_permissions_to_attest = 0;

    while (idx_permissions_to_attest < len_permissions_to_attest) {
      let accreditation = &self.permissions_to_attest[idx_permissions_to_attest];
      let maybe_property_constraint = accreditation.trusted_properties.try_get(property_name) ;

      if ( maybe_property_constraint.is_none()) {
        continue
      };
      if (maybe_property_constraint.borrow().matches_property(property_name, property_value)) {
        return true
      };
      idx_permissions_to_attest = idx_permissions_to_attest + 1;
    };


    return false
  }
}
