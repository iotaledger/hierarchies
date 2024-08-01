// HTF Notary module
module htf::main {
  use std::string::String;
  use sui::vec_map::{Self, VecMap};
  use sui::tx_context::{Self, TxContext};
  use sui::event;
  use sui::vec_set::{Self, VecSet};

  use htf::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use htf::trusted_constraint::{Self, TrustedPropertyConstraints, TrustedPropertyConstraint};
  use htf::permission_to_attest::{Self, PermissionsToAttest};
  use htf::permission_to_accredit::{Self, PermissionsToAccredit};
  use htf::permission::{Self, Permissions};

  const  EUnauthorizedWrongFederation  : u64  = 1;
  const  EUnauthorizedInsufficientAccreditation : u64 = 2;
  const  EUnauthorizedInsufficientAttestation : u64 = 3;
  const  EInvalidProperty: u64 = 4;
  const  EInvalidIssuer: u64 = 5;
  const  EInvalidIssuerInsufficientAttestation: u64 = 6;
  const  EInvalidConstraint  : u64 = 7;
  const  EInvalidTimeSpan: u64 = 8;
  const  ECredentialRevoked: u64 = 9;
  const  EPermissionNotFound: u64 = 10;

  // Federation is the hierarcy of trust in the system. Itsa a public, shared object
  public struct Federation has store, key {
    id : UID,
    governance:        Governance,
    root_authorities:  vector<RootAuthority>,
  }

// Root authority has the highest trust in the system, it can delegate trust to other entities and itself
  public struct RootAuthority  has store, key{
    id : UID,
    account_id: String,
    permissions: Permissions,
  }


  // Governance defines contains a trust hierhchy base
  public struct Governance has store, key {
    id : UID,
    // Trusted Properties all are properties that are trusted by the Federation
    trusted_constraints : TrustedPropertyConstraints,
    // user-id => permission_to_accredit
    accreditors : VecMap<ID, PermissionsToAccredit>,
    // trusted_delegate_id => attestation
    attesters : VecMap<ID, PermissionsToAttest>,
    // owener id ->
    credentials_state : VecMap<ID, CredentialState>
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
    issued_for : ID,
    valid_from : u64,
    valid_until : u64,
    trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>,
  }

  public struct CredentialState has store {
    is_revoked : bool,
  }

  fun new_credential_state() : CredentialState {
    CredentialState {
      is_revoked : false,
    }
  }


  public fun new_federation(ctx :&mut TxContext)  {
    let federation_id = object::new(ctx);
    let mut federation = Federation {
      id : federation_id,
      root_authorities : vector[],
      governance : Governance {
        id : object::new(ctx),
        trusted_constraints : trusted_constraint::new_trusted_property_constraints(),
        accreditors : vec_map::empty(),
        attesters : vec_map::empty(),
        credentials_state : vec_map::empty(),
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

  public fun has_federation_property(self : &Federation, property_name : TrustedPropertyName) : bool {
     self.governance.trusted_constraints.data().contains(&property_name)
  }

  fun find_permissions_to_attest(self: &Federation, user_id : &ID)  :  &PermissionsToAttest {
      self.governance.attesters.get(user_id)
  }


  public fun has_permissions_to_attest(self : &Federation, user_id : &ID)  : bool {
    self.governance.attesters.contains(user_id)
  }

  fun find_permissions_to_accredit(self : &Federation, user_id : &ID) : &PermissionsToAccredit {
    self.governance.accreditors.get(user_id)
  }


  public fun has_permissions_to_accredit(self : &Federation, user_id :&ID)  : bool {
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

  public fun add_root_authority(
      cap : &RootAuthorityCap,
      federation : &mut Federation,
      account_id : String,
      ctx : &mut TxContext,
    ) {
    if  (cap.federation_id != federation.federation_id()) {
      abort EUnauthorizedWrongFederation
    };

    let root_authority = Self::new_root_authority(account_id, ctx);
    vector::push_back(&mut federation.root_authorities, root_authority);
  }

  fun new_root_authority_cap(federation : &Federation, ctx : &mut TxContext )  : RootAuthorityCap {
    RootAuthorityCap {
      id : object::new(ctx),
      federation_id: federation.federation_id()
    }
  }

  fun new_root_authority(account_id: String, ctx: &mut TxContext)  : RootAuthority {
    RootAuthority {
      id : object::new(ctx),
      account_id : account_id,
      permissions : permission::empty(ctx),
    }
  }


  /// Issue an accredidation to accredit about given trusted properties
  public fun issue_permission_to_accredit(cap : &AccreditCap,  federation : &mut Federation, receiver : ID, want_property_constraints : vector<TrustedPropertyConstraint>,  ctx : &mut TxContext) {
      assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);

      let permissions_to_accredit = federation.find_permissions_to_accredit(&ctx.sender().to_id());
      assert!(permissions_to_accredit.are_constraints_permitted(&want_property_constraints), EUnauthorizedInsufficientAccreditation);

      let mut trusted_constraints :VecMap<TrustedPropertyName, TrustedPropertyConstraint> =  vec_map::empty();
      let want_property_constraints_len = vector::length<TrustedPropertyConstraint>(&want_property_constraints);
      let mut idx = 0;
      while (idx < want_property_constraints_len ) {
        trusted_constraints.insert(*want_property_constraints[idx].property_name(), want_property_constraints[idx]);
        idx = idx + 1;
      };


      let permission = permission_to_accredit::new_permission_to_accredit(federation.federation_id(), trusted_constraints, ctx);
      if ( federation.governance.accreditors.contains(&receiver) ) {
          federation.governance.accreditors.get_mut(&receiver).add(permission);
        } else {
          let mut permissions_to_accredit  = permission_to_accredit::new_permissions_to_accredit();
          permissions_to_accredit.add(permission);
          federation.governance.accreditors.insert(receiver, permissions_to_accredit);

          // also create a capability
          transfer::transfer(federation.new_cap_accredit(ctx), receiver.to_address());
        }
  }

  /// creates a permission  (permission_to_attest) to attest about attributes
  public fun issue_permission_to_attest(cap : &AttestCap, federation : &mut Federation, receiver : ID, wanted_constraints: vector<TrustedPropertyConstraint>, ctx : &mut TxContext) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);

    let permissions_to_accredit = federation.find_permissions_to_accredit(&ctx.sender().to_id());
    assert!(permissions_to_accredit.are_constraints_permitted(&wanted_constraints), EUnauthorizedInsufficientAccreditation);

    let permission = permission_to_attest::new_permission_to_attest(
      federation.federation_id(), trusted_constraint::to_map_of_constraints(wanted_constraints), ctx
    );

    if ( federation.governance.attesters.contains(&receiver))  {
      federation.governance.attesters.get_mut(&receiver).add_permission_to_attest(permission);
    } else {
        let mut permissions_to_attest = permission_to_attest::new_permissions_to_attest();
        permissions_to_attest.add_permission_to_attest(permission);
        federation.governance.attesters.insert(receiver, permissions_to_attest);

        // also create a capability
        transfer::transfer(federation.new_cap_attest(ctx), receiver.to_address());
    };
  }

  public fun revoke_permission_to_attest(cap : &AttestCap, federation : &mut Federation, user_id : &ID,  permission_id : &ID, ctx : &mut TxContext) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);

    let remover_permissions = federation.find_permissions_to_attest(&ctx.sender().to_id());

    let users_attest_permissions = federation.find_permissions_to_attest(user_id);
    let mut permission_to_revoke_idx = users_attest_permissions.find_permission_idx(permission_id);
    assert!(permission_to_revoke_idx.is_some(), EPermissionNotFound);

    // Make suere that the sender has the right to revoke the permission
    let permission_to_revoke = &users_attest_permissions.permisssions()[permission_to_revoke_idx.extract()];
    let (_, constraints) = (*permission_to_revoke.constraints()).into_keys_values() ;
    assert!(remover_permissions.are_constraints_permitted(&constraints), EUnauthorizedInsufficientAttestation);

    // Remove the permission
    let users_attest_permissions =  federation.governance.attesters.get_mut(user_id);
    users_attest_permissions.remove_permission(permission_id);
  }


  public fun revoke_permission_to_accredit(cap : &AccreditCap, federation : &mut Federation, user_id : &ID,  permission_id : &ID, ctx : &mut TxContext) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);

    let remover_permissions = federation.find_permissions_to_accredit(&ctx.sender().to_id());

    let users_accredit_permissions = federation.find_permissions_to_accredit(user_id);
    let mut permission_to_revoke_idx = users_accredit_permissions.find_permission_idx(permission_id);
    assert!(permission_to_revoke_idx.is_some(), EPermissionNotFound);

    // make suere that the sender has the right to revoke the permission
    let permission_to_revoke = &users_accredit_permissions.permisssions()[permission_to_revoke_idx.extract()];
    let (_, constraints) = (*permission_to_revoke.constriants()).into_keys_values() ;
    assert!(remover_permissions.are_constraints_permitted(&constraints), EUnauthorizedInsufficientAccreditation);

    // Remove the permission
    let users_accredit_permissions =  federation.governance.accreditors.get_mut(user_id);
    users_accredit_permissions.remove_permission(permission_id);
  }

  public fun issue_credential(
      cap : &AttestCap,
      federation : &mut Federation,
      receiver : ID,
      trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>,
      valid_from_ts_ms : u64,
      valid_until_ts_ms : u64,
      ctx : &mut TxContext)  {
      assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);

      let permissions_to_attest = federation.find_permissions_to_attest(&ctx.sender().to_id());
      assert!(permissions_to_attest.are_values_permitted(&trusted_properties), EUnauthorizedInsufficientAttestation);

      let creds = new_credential(trusted_properties, valid_from_ts_ms, valid_until_ts_ms, receiver, ctx);
      federation.governance.credentials_state.insert(creds.id.to_inner(), new_credential_state());

      transfer::transfer(creds, receiver.to_address());
  }

  public fun validate_credential(self : &Federation, credential : &Credential, ctx : &mut TxContext) {
    assert!(
      self.governance.trusted_constraints.are_properties_correct(credential.trusted_properties()),
      EInvalidProperty,
    );
    assert!(
      self.has_permissions_to_accredit(credential.issued_by()),
      EInvalidIssuer,
    );
    assert!(
      !self.is_credential_revoked(&credential.id.to_inner()),
      ECredentialRevoked,
    );

    let current_time_ms = ctx.epoch_timestamp_ms();
    assert!(
      credential.valid_from <= current_time_ms && credential.valid_until >= current_time_ms,
      EInvalidTimeSpan,
    );

    let issuer_permissions_to_attest = self.find_permissions_to_attest(credential.issued_by());
    assert!(
      issuer_permissions_to_attest.are_values_permitted(credential.trusted_properties()),
      EInvalidIssuerInsufficientAttestation,
    );
  }


  fun new_credential(
    trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>,
    valid_from_ts_ms : u64,
    valid_until_ts_ms : u64,
    issued_for : ID,
    ctx : &mut TxContext,
  ) : Credential {
      Credential {
        id : object::new(ctx),
        issued_by : ctx.sender().to_id(),
        issued_for,
        trusted_properties,
        valid_from:  valid_from_ts_ms,
        valid_until: valid_until_ts_ms,
      }
  }

  fun issued_by(self : &Credential)  : &ID {
    &self.issued_by
  }

  fun trusted_properties(self : &Credential) :  &VecMap<TrustedPropertyName, TrustedPropertyValue> {
    &self.trusted_properties
  }

  fun is_credential_revoked(self : &Federation, credential_id : &ID) : bool {
    if (!self.governance.credentials_state.contains(credential_id)) {
      return false
    };
    self.governance.credentials_state.get(credential_id).is_revoked
  }
}
