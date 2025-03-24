// HTF Notary module
module ith::permission {

  use std::string::String;
  use iota::vec_map::VecMap;

  use ith::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use ith::trusted_constraint::{TrustedPropertyConstraint};
  use ith::utils;

  public struct Permissions has store {
    permissions : vector<Permission>,
  }

  public(package) fun new_permissions() : Permissions {
    Permissions {
      permissions: vector::empty(),
    }
  }

  /// Permission can be created only by the HTF module
  public struct Permission has store, key {
    // TODO fixme when auditit the security model
    id : UID,
    federation_id : ID,
    created_by : String,
    constraints : VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
  }

  public(package) fun new_permission(federation_id: ID, constraints: VecMap<TrustedPropertyName, TrustedPropertyConstraint>, ctx : &mut TxContext) : Permission {
    Permission {
        id : object::new(ctx),
        federation_id,
        created_by : ctx.sender().to_string(),
        constraints: constraints,
    }
  }

  public(package) fun id(self : &Permission) : &UID {
    &self.id
  }

  public(package) fun new_empty_permission(): Permissions {
    Permissions {
      permissions: vector::empty(),
    }
  }

  public(package)  fun federation_id(self : &Permission) : &ID {
    &self.federation_id
  }

  public(package) fun created_by(self : &Permission) : &String {
    &self.created_by
  }

  public(package) fun constraints(self : &Permission) : &VecMap<TrustedPropertyName, TrustedPropertyConstraint> {
    &self.constraints
  }

  public(package) fun add_permission_to_attest(self : &mut Permissions, permission_to_attest : Permission) {
    self.permissions.push_back(permission_to_attest);
  }

  /// checks if all constraints matches the given in Accredidations.
  public(package) fun are_values_permitted(self : &Permissions, trusted_properties: &VecMap<TrustedPropertyName, TrustedPropertyValue>, current_time_ms : u64) :bool {
    let property_names = trusted_properties.keys() ;
    let mut idx_property_names = 0;

    while ( idx_property_names < property_names.length() ) {
      let property_name = property_names[idx_property_names];
      let property_value = trusted_properties.get(&property_name);

      if (!self.is_value_permitted(&property_name, property_value, current_time_ms)  ) {
        return false
      };

      idx_property_names = idx_property_names + 1;
    };

    return true
  }


  public(package) fun is_value_permitted(self : &Permissions, property_name : &TrustedPropertyName, property_value : &TrustedPropertyValue, current_time_ms : u64) :  bool {
    let len_permissions_to_attest = self.permissions.length();
    let mut idx_permissions_to_attest = 0;

    while (idx_permissions_to_attest < len_permissions_to_attest) {
      let accreditation = &self.permissions[idx_permissions_to_attest];
      let maybe_property_constraint = accreditation.constraints.try_get(property_name) ;

      if ( maybe_property_constraint.is_none()) {
        continue
      };
      if (maybe_property_constraint.borrow().matches_property(property_name, property_value, current_time_ms)) {
        return true
      };
      idx_permissions_to_attest = idx_permissions_to_attest + 1;
    };


    return false
  }

  public(package) fun are_constraints_permitted(self : &Permissions, constraints: &vector<TrustedPropertyConstraint>, current_time_ms : u64 ) :bool {
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

  public(package) fun is_constraint_permitted(self : &Permissions, constraint : &TrustedPropertyConstraint, current_time_ms : u64) :  bool {
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
        if ( maybe_property_constraint.borrow().matches_value(&constraint_value, current_time_ms) ) {
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

    public(package) fun add(self : &mut Permissions, permission : Permission) {
    self.permissions.push_back(permission);
  }

  public(package)  fun permisssions(self : &Permissions) : &vector<Permission> {
    &self.permissions
  }

  public(package) fun remove_permission(self : &mut Permissions, id : &ID) {
    let mut idx = self.find_permission_idx(id);
    if (idx.is_none()) {
      return
    };
    let Permission {
      id,
      constraints : _,
      federation_id : _,
      created_by : _,
     } = self.permissions.remove(idx.extract());
    object::delete(id);
  }

  public(package) fun find_permission_idx(self : &Permissions, id : &ID) : Option<u64> {
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
