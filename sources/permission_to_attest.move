// HTF Notary module
module htf::permission_to_attest {

  use std::string::String;
  use iota::vec_map::{Self, VecMap};

  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use htf::trusted_constraint::{TrustedPropertyConstraint};
  use htf::utils;

  public struct PermissionsToAttest has store {
    permissions : vector<PermissionToAttest>,
  }

  /// PermissionToAttest can be created only by the HTF module
  public struct PermissionToAttest has store {
    id : UID,
    federation_id : ID,
    created_by : String,
    constraints : VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
  }

  public(package) fun new_permission_to_attest(federation_id: ID, constraints: VecMap<TrustedPropertyName, TrustedPropertyConstraint>, ctx : &mut TxContext) : PermissionToAttest {
    PermissionToAttest {
        id : object::new(ctx),
        federation_id,
        created_by : ctx.sender().to_string(),
        constraints: constraints,
    }
  }

  public(package) fun id(self : &PermissionToAttest) : &UID {
    &self.id
  }

  public(package) fun new_permissions_to_attest(): PermissionsToAttest {
    PermissionsToAttest {
      permissions: vector::empty(),
    }
  }

  public(package)  fun federation_id(self : &PermissionToAttest) : &ID {
    &self.federation_id
  }

  public(package) fun created_by(self : &PermissionToAttest) : &String {
    &self.created_by
  }

  public(package) fun constraints(self : &PermissionToAttest) : &VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &self.constraints
  }

  public(package) fun add_permission_to_attest(self : &mut PermissionsToAttest, permission_to_attest : PermissionToAttest) {
    self.permissions.push_back(permission_to_attest);
  }

  /// checks if all constraints matches the given in Accredidations
  public(package) fun are_values_permitted(self : &PermissionsToAttest, trusted_properties: &VecMap<TrustedPropertyName, TrustedPropertyValue> ) :bool {
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


  public(package) fun is_value_permitted(self : &PermissionsToAttest, property_name : &TrustedPropertyName, property_value : &TrustedPropertyValue) :  bool {
    let len_permissions_to_attest = self.permissions.length();
    let mut idx_permissions_to_attest = 0;

    while (idx_permissions_to_attest < len_permissions_to_attest) {
      let accreditation = &self.permissions[idx_permissions_to_attest];
      let maybe_property_constraint = accreditation.constraints.try_get(property_name) ;

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

  public(package) fun are_constraints_permitted(self : &PermissionsToAttest, constraints: &vector<TrustedPropertyConstraint> ) :bool {
    let mut idx = 0;
    while ( idx < constraints.length()  ) {
        let constraint = constraints[idx];
        if ( ! self.is_constraint_permitted(&constraint)  )  {
          return false
        };
        idx = idx + 1;
    };
    return true
  }

  public(package) fun is_constraint_permitted(self : &PermissionsToAttest, constraint : &TrustedPropertyConstraint) :  bool {
    let len_permissions = self.permissions.length();
    let mut idx_permissions = 0;
    let mut want_constraints : vector<TrustedPropertyValue> = utils::copy_vector(constraint.allowed_values().keys());

    while (idx_permissions < len_permissions) {
      let permission = &self.permissions[idx_permissions];

      let maybe_property_constraint = permission.constraints.try_get(constraint.property_name()) ;
      if ( maybe_property_constraint.is_none()) {
        continue
      };

      let mut len_want_constraints = want_constraints.length();
      let mut idx_want_constraints = 0;
      while (idx_want_constraints < len_want_constraints ) {
        let constraint_value = want_constraints[idx_want_constraints];
        if ( maybe_property_constraint.borrow().matches_value(&constraint_value) ) {
          want_constraints.remove(idx_want_constraints);
          len_want_constraints = len_want_constraints - 1;
        };
        idx_want_constraints = idx_want_constraints + 1;
      };
      idx_permissions = idx_permissions + 1;
    };

    // alll wanted constraints have been found
    if (want_constraints.length() == 0 ) {
      return true
    };

    return false
  }

  public(package)  fun permisssions(self : &PermissionsToAttest) : &vector<PermissionToAttest> {
    &self.permissions
  }

  public(package) fun remove_permission(self : &mut PermissionsToAttest, id : &ID) {
    let mut idx = self.find_permission_idx(id);
    if (idx.is_none()) {
      return
    };
    let PermissionToAttest {
      id,
      constraints : _,
      federation_id : _,
      created_by : _,
     } = self.permissions.remove(idx.extract());
    object::delete(id);
  }

  public(package) fun find_permission_idx(self : &PermissionsToAttest, id : &ID) : Option<u64> {
    let mut idx = 0;
    while (idx < self.permissions.length()) {
      if (self.permissions[idx].id.to_inner() == *id) {
        return option::some(idx)
      };
      idx = idx + 1;
    };
    option::none()
  }
}
