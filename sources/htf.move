// HTF Notary module
module htf::main {

  use std::string::String;
  use sui::object;
  use sui::vec_map::{Self, VecMap};
  use sui::table::{Self, Table};
  use sui::tx_context::{Self, TxContext};
  use sui::event;
  use sui::vec_set::{Self, VecSet};

  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use htf::trusted_constraint::{Self, TrustedPropertyConstraints, TrustedPropertyConstraint};
  use htf::credential::{Self, Credential};
  use htf::permission_to_atest::{Self, PersmissionsToAtest};
  use htf::permission_to_acredit::{Self, PermissionsToAcredit};
  use htf::permission::{Self, Permissions};
  use htf::trusted_service::{Self, TrustedService};
  use htf::utils;


 const  E_UNAUTHORIZED_WRONG_FEDERATION  : u64  = 1;
 const  E_UNAUTHORIZED_INSUFFICIENT_ACCREDITATION : u64 = 2;
 const  E_UNAUTHORIZED_INSUFFICIENT_ATESTATION : u64 = 3;
 const  E_INVALID_PROPERTY: u64 = 3;
 const  E_INVALID_ISSUER: u64 = 4;
 const  E_INVALID_ISSUER_INSUFFICIENT_ATESTATION: u64 = 4;
 const  E_INVALID_CONSTRAINT  : u64 = 5;

  // Federation is the hierarcy of trust in the system. Itsa a public, shared object
  public struct Federation has store, key {
    id : UID,
    governance:        Governance,
    root_authorities:  vector<RootAuthority>,
    trust_services:    Table<String, TrustedService>,
  }

// Root authority has the highest trust in the system, it can delegate trust to other entities and itself
  public struct RootAuthority  has store, key{
    id : UID,
    trust_service: String,
    id_in_trust_service : String,
    permissions : Permissions,
  }


  // Governance defines contains a trust hierhchy base
  public struct Governance has store, key {
    id : UID,
    // Trusted Properties all are properties that are trusted by the Federation
    trusted_constraints : TrustedPropertyConstraints,
    // user-id => permission_to_acredit
    issued_permissions_to_acredit : Table<ID, PermissionsToAcredit>,
    // trusted_delegate_id => attestation
    issued_permissions_to_atest : Table<ID, PersmissionsToAtest>,
  }


  public struct RootAuthorityCap has key { id : UID, federation_id : String }
  public struct AttestCap has key { id : UID, federation_id : String, }
  public struct AccreditCap has key { id: UID, federation_id : String}


  public struct Event<D> has copy, drop {
    data : D,
  }

  public struct FederationCreatedEvent has copy, drop {
    federation_address : String,
  }

  public fun new_federation(ctx :&mut TxContext)  {
    let federation_id = object::new(ctx);
    let federation_address = federation_id.to_address().to_string();
    let mut federation = Federation {
      id : federation_id,
      trust_services : table::new(ctx),
      root_authorities : vector[],
      governance : Governance {
        id : object::new(ctx),
        trusted_constraints : trusted_constraint::new_trusted_property_constraints(),
        issued_permissions_to_acredit : table::new(ctx),
        issued_permissions_to_atest : table::new(ctx),
      },
    };
    let cap = Self::new_root_authority_cap(&federation, ctx);

    // add the root auhtority and the trust service
    Self::add_root_authority(&cap, &mut federation, ctx.sender().to_string(),  ctx);

    event::emit(Event{data: FederationCreatedEvent{
      federation_address: federation_address
      }
    });
    transfer::transfer(cap, ctx.sender());
    transfer::share_object(federation)
  }


  fun federation_id(self : &Federation) : String {
    self.id.to_address().to_string()
  }

  fun find_permissions_to_atest(self : &Federation, user_id : ID)  :  &PersmissionsToAtest {
      self.governance.issued_permissions_to_atest.borrow(user_id)
  }

  fun has_permissions_to_atest(self :&Federation, user_id :ID)  : bool {
    self.governance.issued_permissions_to_atest.contains(user_id)
  }

  fun find_permissions_to_acredit(self : &Federation, user_id : ID) : &PermissionsToAcredit {
    self.governance.issued_permissions_to_acredit.borrow(user_id)
  }

  fun has_permissions_to_acredit(self : &Federation, user_id :ID)  : bool {
    self.governance.issued_permissions_to_acredit.contains(user_id)
  }


  /// adds the trusted property to the federation, optionally a specifc type can be given
  public fun add_trusted_property(
    cap : &RootAuthorityCap,
    federation : &mut Federation,
    property_name : TrustedPropertyName,
    allowed_values : VecSet<TrustedPropertyValue>,
    allow_any : bool,
    _ctx : &mut TxContext)
  {
    if  (cap.federation_id != federation.federation_id()) {
      abort E_UNAUTHORIZED_WRONG_FEDERATION
    };
    if (allow_any && allowed_values.keys().length() > 0 ) {
     abort E_INVALID_CONSTRAINT
    };

    let constraint = trusted_constraint::new_trusted_property_constraint(
      property_name,
      allowed_values,
      allow_any,
    );

    federation.governance.trusted_constraints.add_constraint(property_name, constraint) ;
  }

  /// Creates a new accredit capability
  fun new_cap_accredit(self : &Federation, ctx : &mut TxContext) : AccreditCap {
    AccreditCap {
      id : object::new(ctx),
      federation_id : self.federation_id(),
    }
  }

  /// Creates a new attest capability
  fun new_cap_atest(self : &Federation, ctx : &mut TxContext) : AttestCap {
    AttestCap {
      id : object::new(ctx),
      federation_id : self.federation_id(),
    }
  }

  fun add_root_authority(
      cap : &RootAuthorityCap,
     federation : &mut Federation,
      account_id : String,
      ctx : &mut TxContext,
    ) {
    if  (cap.federation_id != federation.federation_id()) {
      abort E_UNAUTHORIZED_WRONG_FEDERATION
    };

    let root_authority = Self::new_root_authority(federation, account_id, ctx);
    vector::push_back(&mut federation.root_authorities, root_authority);
  }

  fun new_root_authority_cap(federation : &Federation, ctx : &mut TxContext )  : RootAuthorityCap {
    RootAuthorityCap {
      id : object::new(ctx),
      federation_id: federation.id.to_address().to_string()
    }
  }

  fun new_root_authority(federation : &mut Federation, account_id : String, ctx : &mut TxContext)  : RootAuthority {
    Self::add_trust_service(federation, b"account".to_string(), ctx);

    RootAuthority {
      id : object::new(ctx),
      trust_service:  b"account".to_string(),
      id_in_trust_service : account_id,
      permissions : permission::empty(ctx),
    }
  }

  fun add_trust_service(federation : &mut Federation,  service_type : String, ctx :&mut TxContext) {
    if ( table::contains(&federation.trust_services, service_type) ) {
      return
    };
    let trust_service = trusted_service::new_trust_service(ctx, service_type);
    table::add(&mut federation.trust_services, service_type, trust_service);
  }

  /// Issue an accredidation to accredit about given trusted properties
  public fun issue_permission_to_acredit(cap : &AccreditCap,  federation : &mut Federation, receiver : ID, want_property_constraints : vector<TrustedPropertyConstraint>,  ctx : &mut TxContext) {
      let federation_id = federation.id.to_address().to_string();
      if (cap.federation_id !=  federation_id)   {
        abort E_UNAUTHORIZED_WRONG_FEDERATION
      };
      let permissions_to_acredit = federation.find_permissions_to_acredit(ctx.sender().to_id());
      if (! permissions_to_acredit.are_constraints_permitted(&want_property_constraints)) {
        abort E_UNAUTHORIZED_INSUFFICIENT_ACCREDITATION
      };

      let mut trusted_constraints :VecMap<TrustedPropertyName, TrustedPropertyConstraint> =  vec_map::empty();
      let want_property_constraints_len = vector::length<TrustedPropertyConstraint>(&want_property_constraints);
      let mut idx = 0;
      while (idx < want_property_constraints_len ) {
        trusted_constraints.insert(*want_property_constraints[idx].property_name(), want_property_constraints[idx]);
        idx = idx + 1;
      };


      let permission = permission_to_acredit::new_permission_to_acredit(federation_id, trusted_constraints, ctx);
      if ( federation.governance.issued_permissions_to_acredit.contains(receiver) ) {
          federation.governance.issued_permissions_to_acredit.borrow_mut(receiver).add(permission);
        } else {
          let mut permissions_to_acredit  = permission_to_acredit::new_permissions_to_acredit();
          permissions_to_acredit.add(permission);
          federation.governance.issued_permissions_to_acredit.add(receiver, permissions_to_acredit);

          // also create a capability
          transfer::transfer(federation.new_cap_accredit(ctx), receiver.to_address());
        }
  }

  /// creates a permission  (permission_to_atest) to atest about attributes
  public fun issue_permission_to_atest(cap : &AttestCap, federation : &mut Federation, receiver : ID, wanted_constraints: vector<TrustedPropertyConstraint>, ctx : &mut TxContext) {
    let federation_id = federation.id.to_address().to_string();

    if (cap.federation_id !=  federation_id)   {
      abort E_UNAUTHORIZED_WRONG_FEDERATION
    };
    let permissions_to_acredit = federation.find_permissions_to_acredit(ctx.sender().to_id());
    if (! permissions_to_acredit.are_constraints_permitted(&wanted_constraints)) {
      abort E_UNAUTHORIZED_INSUFFICIENT_ACCREDITATION
    };

    let permission = permission_to_atest::new_permission_to_atest(
      federation_id, trusted_constraint::to_map_of_constraints(wanted_constraints), ctx
    );

    if ( federation.governance.issued_permissions_to_atest.contains(receiver))  {
      federation.governance.issued_permissions_to_atest.borrow_mut(receiver).add_permission_to_atest(permission);
    } else {
        let mut permissions_to_atest = permission_to_atest::new_permissions_to_atest();
        permissions_to_atest.add_permission_to_atest(permission);
        federation.governance.issued_permissions_to_atest.add(receiver, permissions_to_atest);

        // also create a capability
        transfer::transfer(federation.new_cap_atest(ctx), receiver.to_address());
    };
  }

  public fun issue_credential(cap : &AttestCap, federation : &mut Federation, receiver : ID,  trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>,  ctx : &mut TxContext)  {
      let federation_id = federation.id.to_address().to_string();
      if (cap.federation_id !=  federation_id)   {
        abort E_UNAUTHORIZED_WRONG_FEDERATION
      };
      let permissions_to_atest = federation.find_permissions_to_atest(ctx.sender().to_id());
      if (!permissions_to_atest.are_values_permitted(&trusted_properties))  {
        abort E_UNAUTHORIZED_INSUFFICIENT_ATESTATION
      };

      let creds = credential::new(trusted_properties, ctx);
      transfer::public_transfer(creds, receiver.to_address());
  }

  public fun validate_credential(self:  &Federation, credential : &Credential) {
    if (! self.governance.trusted_constraints.are_properties_correct(credential.trusted_properties())) {
      abort E_INVALID_PROPERTY
    };

    if (! self.has_permissions_to_atest(*credential.issued_by()) ) {
      abort E_INVALID_ISSUER
    };

    let issuer_permissions_to_atest = self.find_permissions_to_atest(*credential.issued_by());
    if (! issuer_permissions_to_atest.are_values_permitted(credential.trusted_properties())) {
      abort E_INVALID_ISSUER_INSUFFICIENT_ATESTATION
    } ;
  }

}
