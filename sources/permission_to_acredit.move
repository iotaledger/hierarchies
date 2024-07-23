// HTF Notary module
module htf::permission_to_acredit {

  use std::string::String;
  use sui::vec_map::{Self, VecMap};

  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use htf::trusted_constraint::{TrustedPropertyConstraint};
  use htf::utils;

  public struct PermissionsToAcredit has store {
    permissions_to_acredit  : vector<PermissionToAcredit>,
  }

  // Accredidation can be created only by the HTF module
  public struct PermissionToAcredit has store, key {
    id : UID,
    federation_id : ID,
    created_by : String,
    trusted_constraints : VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
  }

  public(package) fun new_permission_to_acredit(federation_id  : ID, constraints : VecMap<TrustedPropertyName, TrustedPropertyConstraint>, ctx : &mut TxContext)  : PermissionToAcredit {
    PermissionToAcredit {
      id : object::new(ctx),
      federation_id,
      trusted_constraints: constraints,
      created_by : ctx.sender().to_string(),
    }
  }

  public(package) fun new_permissions_to_acredit() : PermissionsToAcredit {
    PermissionsToAcredit {
      permissions_to_acredit: vector::empty(),
    }
  }

  public(package) fun are_constraints_permitted(self : &PermissionsToAcredit, constraints: &vector<TrustedPropertyConstraint> ) :bool {
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


  public(package) fun is_constraint_permitted(self : &PermissionsToAcredit, constraint : &TrustedPropertyConstraint) :  bool {
    let len_permissions_to_acredit = self.permissions_to_acredit.length();
    let mut idx_permissions_to_acredit = 0;
    let mut want_constraints : vector<TrustedPropertyValue> = utils::copy_vector(constraint.allowed_values().keys());

    while (idx_permissions_to_acredit < len_permissions_to_acredit) {
      let permission = &self.permissions_to_acredit[idx_permissions_to_acredit];

      let maybe_property_constraint = permission.trusted_constraints.try_get(constraint.property_name()) ;
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
      idx_permissions_to_acredit = idx_permissions_to_acredit + 1;
    };

    // alll wanted constraints have been found
    if (want_constraints.length() == 0 ) {
      return true
    };

    return false
  }

  public(package) fun add(self : &mut PermissionsToAcredit, permission_to_acredit : PermissionToAcredit) {
    self.permissions_to_acredit.push_back(permission_to_acredit);
  }

}
