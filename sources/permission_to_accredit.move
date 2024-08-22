// HTF Notary module
module htf::permission_to_accredit {

  use std::string::String;
  use iota::vec_map::VecMap;

  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use htf::trusted_constraint::{TrustedPropertyConstraint};
  use htf::utils;

  public struct PermissionsToAccredit has store {
    permissions  : vector<PermissionToAccredit>,
  }

  // Accredidation can be created only by the HTF module
  public struct PermissionToAccredit has store, key {
    id : UID,
    federation_id : ID,
    created_by : String,
    constraints : VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
  }

  public(package) fun id(self : &PermissionToAccredit) : &UID {
    &self.id
  }


  public(package) fun constriants(self : &PermissionToAccredit) : &VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &self.constraints
  }

  public(package) fun new_permission_to_accredit(federation_id  : ID, constraints : VecMap<TrustedPropertyName, TrustedPropertyConstraint>, ctx : &mut TxContext)  : PermissionToAccredit {
    PermissionToAccredit {
      id : object::new(ctx),
      federation_id,
      constraints: constraints,
      created_by : ctx.sender().to_string(),
    }
  }

  public(package) fun new_permissions_to_accredit() : PermissionsToAccredit {
    PermissionsToAccredit {
      permissions: vector::empty(),
    }
  }

  public(package) fun are_constraints_permitted(self : &PermissionsToAccredit, constraints: &vector<TrustedPropertyConstraint>, current_time_ms : u64) :bool {
    let mut idx = 0;
    while ( idx < constraints.length()  ) {
        let constraint = constraints[idx];
        if ( ! self.is_constraint_permitted(&constraint, current_time_ms)  )  {
          return false
        };
        idx = idx + 1;
    };
    return true
  }


  public(package) fun is_constraint_permitted(self : &PermissionsToAccredit, constraint : &TrustedPropertyConstraint, current_time_ms : u64) :  bool {
    let len_permissions_to_accredit = self.permissions.length();
    let mut idx_permissions_to_accredit = 0;
    let mut want_constraints : vector<TrustedPropertyValue> = utils::copy_vector(constraint.allowed_values().keys());

    while (idx_permissions_to_accredit < len_permissions_to_accredit) {
      let permission = &self.permissions[idx_permissions_to_accredit];

      let maybe_property_constraint = permission.constraints.try_get(constraint.property_name()) ;
      if ( maybe_property_constraint.is_none()) {
        continue
      };

      let mut len_want_constraints = want_constraints.length();
      let mut idx_want_constraints = 0;
      while (idx_want_constraints < len_want_constraints ) {
        let constraint_value = want_constraints[idx_want_constraints];
        if ( maybe_property_constraint.borrow().matches_value(&constraint_value, current_time_ms) ) {
          want_constraints.remove(idx_want_constraints);
          len_want_constraints = len_want_constraints - 1;
        };
        idx_want_constraints = idx_want_constraints + 1;
      };
      idx_permissions_to_accredit = idx_permissions_to_accredit + 1;
    };

    // alll wanted constraints have been found
    if (want_constraints.length() == 0 ) {
      return true
    };

    return false
  }

  public(package) fun add(self : &mut PermissionsToAccredit, permission_to_accredit : PermissionToAccredit) {
    self.permissions.push_back(permission_to_accredit);
  }

  public(package)  fun permisssions(self : &PermissionsToAccredit) : &vector<PermissionToAccredit> {
    &self.permissions
  }

  public(package) fun remove_permission(self : &mut PermissionsToAccredit, id : &ID) {
    let mut idx = self.find_permission_idx(id);
    if (idx.is_none()) {
      return
    };
    let PermissionToAccredit {
      id,
      constraints : _,
      federation_id : _,
      created_by : _,
     } = self.permissions.remove(idx.extract());
    object::delete(id);
  }

  public(package) fun find_permission_idx(self : &PermissionsToAccredit, id : &ID) : Option<u64> {
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
