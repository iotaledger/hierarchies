// HTF Notary module
module htf::main {
  use std::string::String;
  use sui::vec_map::{Self, VecMap};
  use sui::table::{Self, Table};
  use sui::tx_context::{Self, TxContext};
  use sui::event;
  use sui::vec_set::{Self, VecSet};

  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use htf::trusted_constraint::{Self, TrustedPropertyConstraints, TrustedPropertyConstraint};
  use htf::permission_to_attest::{Self, PersmissionsToAttest};
  use htf::permission_to_accredit::{Self, PermissionsToAccredit};
  use htf::permission::{Self, Permissions};
  use htf::trusted_service::{Self, TrustedService};

  const  EUnauthorizedWrongFederation  : u64  = 1;
  const  EUnauthorizedInsufficientAccreditation : u64 = 2;
  const  EUnauthorizedInsufficientAttestation : u64 = 3;
  const  EInvalidProperty: u64 = 3;
  const  EInvalidIssuer: u64 = 4;
  const  EInvalidIssuerInsufficientAttestation: u64 = 4;
  const  EInvalidConstraint  : u64 = 5;

  // Federation is the hierarcy of trust in the system. Itsa a public, shared object
  public struct Federation has store, key {
    id : UID,
    governance:        Governance,
    root_authorities:  vector<RootAuthority>,
    trust_services:    VecMap<String, TrustedService>,
  }

// Root authority has the highest trust in the system, it can delegate trust to other entities and itself
  public struct RootAuthority  has store, key{
    id : UID,
    trust_service: String,
    id_in_trust_service: String,
    permissions: Permissions,
  }


  // Governance defines contains a trust hierhchy base
  public struct Governance has store, key {
    id : UID,
    // Trusted Properties all are properties that are trusted by the Federation
    trusted_constraints : TrustedPropertyConstraints,
    // user-id => permission_to_accredit
    accreditors : Table<ID, PermissionsToAccredit>,
    // trusted_delegate_id => attestation
    attesters : Table<ID, PersmissionsToAttest>,
  }


  public struct RootAuthorityCap has key { id : UID, federation_id : ID }
  public struct AttestCap has key { id : UID, federation_id : ID, }
  public struct AccreditCap has key { id: UID, federation_id : ID}


  public struct Event<D> has copy, drop {
    data : D,
  }

  public struct FederationCreatedEvent has copy, drop {
    federation_address : address,
  }

  public struct Credential has key {
    id : UID,
    issued_by : ID,
    trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>,
  }

  public fun new_federation(ctx :&mut TxContext)  {
    let federation_id = object::new(ctx);
    let mut federation = Federation {
      id : federation_id,
      trust_services : vec_map::empty(),
      root_authorities : vector[],
      governance : Governance {
        id : object::new(ctx),
        trusted_constraints : trusted_constraint::new_trusted_property_constraints(),
        accreditors : table::new(ctx),
        attesters : table::new(ctx),
      },
    };
    let cap = Self::new_root_authority_cap(&federation, ctx);
    // add the root auhtority and the trust service
    Self::add_root_authority(&cap, &mut federation, ctx.sender().to_string(),  ctx);

    event::emit(Event{data: FederationCreatedEvent{
      federation_address: federation.federation_id().to_address(),
      }
    });
    transfer::transfer(cap, ctx.sender());
    transfer::share_object(federation)
  }


  fun federation_id(self : &Federation) : ID {
    self.id.to_inner()
  }

  fun find_permissions_to_attest(self : &Federation, user_id : ID)  :  &PersmissionsToAttest {
      self.governance.attesters.borrow(user_id)
  }

  fun has_permissions_to_attest(self :&Federation, user_id :ID)  : bool {
    self.governance.attesters.contains(user_id)
  }

  fun find_permissions_to_accredit(self : &Federation, user_id : ID) : &PermissionsToAccredit {
    self.governance.accreditors.borrow(user_id)
  }

  fun has_permissions_to_accredit(self : &Federation, user_id :ID)  : bool {
    self.governance.accreditors.contains(user_id)
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
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);
    assert!(!(allow_any && allowed_values.keys().length() > 0), EInvalidConstraint);

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
  fun new_cap_attest(self : &Federation, ctx : &mut TxContext) : AttestCap {
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
      abort EUnauthorizedWrongFederation
    };

    let root_authority = Self::new_root_authority(federation, account_id, ctx);
    vector::push_back(&mut federation.root_authorities, root_authority);
  }

  fun new_root_authority_cap(federation : &Federation, ctx : &mut TxContext )  : RootAuthorityCap {
    RootAuthorityCap {
      id : object::new(ctx),
      federation_id: federation.federation_id()
    }
  }

  fun new_root_authority(federation: &mut Federation, account_id: String, ctx: &mut TxContext)  : RootAuthority {
    Self::add_trust_service(federation, b"account".to_string(), ctx);

    RootAuthority {
      id : object::new(ctx),
      trust_service:  b"account".to_string(),
      id_in_trust_service : account_id,
      permissions : permission::empty(ctx),
    }
  }

  fun add_trust_service(federation : &mut Federation,  service_type : String, ctx :&mut TxContext) {
    if ( federation.trust_services.contains(&service_type)) {
      return
    };
    let trust_service = trusted_service::new_trust_service(ctx, service_type);
    federation.trust_services.insert(service_type, trust_service);
  }

  /// Issue an accredidation to accredit about given trusted properties
  public fun issue_permission_to_accredit(cap : &AccreditCap,  federation : &mut Federation, receiver : ID, want_property_constraints : vector<TrustedPropertyConstraint>,  ctx : &mut TxContext) {
      assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);

      let permissions_to_accredit = federation.find_permissions_to_accredit(ctx.sender().to_id());
      assert!(permissions_to_accredit.are_constraints_permitted(&want_property_constraints), EUnauthorizedInsufficientAccreditation);

      let mut trusted_constraints :VecMap<TrustedPropertyName, TrustedPropertyConstraint> =  vec_map::empty();
      let want_property_constraints_len = vector::length<TrustedPropertyConstraint>(&want_property_constraints);
      let mut idx = 0;
      while (idx < want_property_constraints_len ) {
        trusted_constraints.insert(*want_property_constraints[idx].property_name(), want_property_constraints[idx]);
        idx = idx + 1;
      };


      let permission = permission_to_accredit::new_permission_to_accredit(federation.federation_id(), trusted_constraints, ctx);
      if ( federation.governance.accreditors.contains(receiver) ) {
          federation.governance.accreditors.borrow_mut(receiver).add(permission);
        } else {
          let mut permissions_to_accredit  = permission_to_accredit::new_permissions_to_accredit();
          permissions_to_accredit.add(permission);
          federation.governance.accreditors.add(receiver, permissions_to_accredit);

          // also create a capability
          transfer::transfer(federation.new_cap_accredit(ctx), receiver.to_address());
        }
  }

  /// creates a permission  (permission_to_attest) to attest about attributes
  public fun issue_permission_to_attest(cap : &AttestCap, federation : &mut Federation, receiver : ID, wanted_constraints: vector<TrustedPropertyConstraint>, ctx : &mut TxContext) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);

    let permissions_to_accredit = federation.find_permissions_to_accredit(ctx.sender().to_id());
    assert!(permissions_to_accredit.are_constraints_permitted(&wanted_constraints), EUnauthorizedInsufficientAccreditation);

    let permission = permission_to_attest::new_permission_to_attest(
      federation.federation_id(), trusted_constraint::to_map_of_constraints(wanted_constraints), ctx
    );

    if ( federation.governance.attesters.contains(receiver))  {
      federation.governance.attesters.borrow_mut(receiver).add_permission_to_attest(permission);
    } else {
        let mut permissions_to_attest = permission_to_attest::new_permissions_to_attest();
        permissions_to_attest.add_permission_to_attest(permission);
        federation.governance.attesters.add(receiver, permissions_to_attest);

        // also create a capability
        transfer::transfer(federation.new_cap_attest(ctx), receiver.to_address());
    };
  }

  public fun issue_credential(cap : &AttestCap, federation : &mut Federation, receiver : ID,  trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>,  ctx : &mut TxContext)  {
      assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);

      let permissions_to_attest = federation.find_permissions_to_attest(ctx.sender().to_id());
      assert!(permissions_to_attest.are_values_permitted(&trusted_properties), EUnauthorizedInsufficientAttestation);

      let creds = new_credential(trusted_properties, ctx);
      transfer::transfer(creds, receiver.to_address());
  }

  public fun validate_credential(self:  &Federation, credential : &Credential) {
    assert!(
      self.governance.trusted_constraints.are_properties_correct(credential.trusted_properties()),
      EInvalidProperty,
    );
    assert!(
      self.has_permissions_to_accredit(*credential.issued_by()),
      EInvalidIssuer,
    );

    let issuer_permissions_to_attest = self.find_permissions_to_attest(*credential.issued_by());
    assert!(
      issuer_permissions_to_attest.are_values_permitted(credential.trusted_properties()),
      EInvalidIssuerInsufficientAttestation,
    );
  }


  fun new_credential(trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>, ctx : &mut TxContext) : Credential {
      Credential {
        id : object::new(ctx),
        issued_by : ctx.sender().to_id(),
        trusted_properties,
      }
  }

  fun issued_by(self : &Credential)  : &ID {
    &self.issued_by
  }

  fun trusted_properties(self : &Credential) :  &VecMap<TrustedPropertyName, TrustedPropertyValue> {
    &self.trusted_properties
  }
}
