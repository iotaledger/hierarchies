// HTF Notary module
module ith::main {
  use iota::vec_map::{Self, VecMap};
  use iota::event;
  use iota::vec_set::{VecSet};

  use ith::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
  use ith::trusted_constraint::{Self, TrustedPropertyConstraints, TrustedPropertyConstraint};
  use ith::permission::{Self, Permissions};

  const  EUnauthorizedWrongFederation  : u64  = 1;
  const  EUnauthorizedInsufficientAccreditation : u64 = 2;
  const  EUnauthorizedInsufficientAttestation : u64 = 3;
  const  EInvalidProperty: u64 = 4;
  const  EInvalidIssuerInsufficientAttestation: u64 = 5;
  const  EInvalidConstraint  : u64 = 6;
  const  EPermissionNotFound: u64 = 7;

  // Federation is the hierarcy of trust in the system. Itsa a public, shared object
  public struct Federation has store, key {
    id : UID,
    governance:        Governance,
    root_authorities:  vector<RootAuthority>,
  }

// Root authority has the highest trust in the system, it can delegate trust to other entities and itself
  public struct RootAuthority  has store, key{
    id : UID,
    account_id: ID,
  }


  // Governance defines contains a trust hierhchy base
  public struct Governance has store, key {
    id : UID,
    // Trusted Properties all are properties that are trusted by the Federation
    trusted_constraints : TrustedPropertyConstraints,
    // user-id => permission_to_accredit
    accreditors : VecMap<ID, Permissions>,
    // trusted_delegate_id => attestation
    attesters : VecMap<ID, Permissions>,
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

  fun is_root_authority(self : &Federation, id : &ID) : bool {
    let mut idx = 0 ;
    while (idx < self.root_authorities.length()) {
      if (self.root_authorities[idx].account_id == id) {
        return true
      };
      idx = idx + 1;
    };
    false
  }


  public fun new_federation(ctx :&mut TxContext)  {
    let federation_id = object::new(ctx);
    let mut federation = Federation {
      id : federation_id,
      root_authorities : vector::empty(),
      governance : Governance {
        id : object::new(ctx),
        trusted_constraints: trusted_constraint::new_trusted_property_constraints(),
        accreditors : vec_map::empty(),
        attesters : vec_map::empty(),
      },
    };

    let root_auth_cap = Self::new_root_authority_cap(&federation, ctx);
    let root_authority = Self::new_root_authority(ctx.sender().to_id(), ctx);
    vector::push_back(&mut federation.root_authorities, root_authority);

    // Add permission to attest
    let permission = permission::new_empty_permission();
    federation.governance.accreditors.insert(ctx.sender().to_id(), permission);

    // Add permission to attest
    let permission = permission::new_empty_permission();
    federation.governance.attesters.insert(ctx.sender().to_id(), permission);

    // Add permission to attest and accredit to the root authority
    let attest_cap = Self::new_cap_attest(&federation, ctx);
    let accredit_cap = Self::new_cap_accredit(&federation, ctx);

    event::emit(Event{data: FederationCreatedEvent{
      federation_address: federation.federation_id().to_address(),
      }
    });

    transfer::transfer(root_auth_cap, ctx.sender());
    transfer::transfer(accredit_cap, ctx.sender());
    transfer::transfer(attest_cap, ctx.sender());

    transfer::share_object(federation)
  }


  fun federation_id(self : &Federation) : ID {
    self.id.to_inner()
  }

  public fun get_trusted_properties(self : &Federation) : vector<TrustedPropertyName> {
    self.governance.trusted_constraints.data().keys()
  }

  public fun is_trusted_property(self : &Federation, property_name : TrustedPropertyName) : bool {
    self.governance.trusted_constraints.data().contains(&property_name)
  }

  public fun get_attestations(self: &Federation, user_id : &ID)  :  &Permissions {
      self.governance.attesters.get(user_id)
  }


  public fun is_attester(self : &Federation, user_id : &ID)  : bool {
    self.governance.attesters.contains(user_id)
  }

  public fun get_accreditations(self : &Federation, user_id : &ID) : &Permissions {
    self.governance.accreditors.get(user_id)
  }


  public fun is_accreditor(self : &Federation, user_id :&ID)  : bool {
    self.governance.accreditors.contains(user_id)
  }


  /// adds the trusted property to the federation, optionally a specifc type can be given
  public fun add_trusted_property(
    self : &mut Federation,
    cap : &RootAuthorityCap,
    property_name : TrustedPropertyName,
    allowed_values : VecSet<TrustedPropertyValue>,
    allow_any : bool,
    _ctx : &mut TxContext)
  {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    assert!(!(allow_any && allowed_values.keys().length() > 0), EInvalidConstraint);

    let constraint = trusted_constraint::new_trusted_property_constraint(
      property_name,
      allowed_values,
      allow_any,
      option::none(),
    );

    self.governance.trusted_constraints.add_constraint(property_name, constraint) ;
  }


  // TODO should be removed
  // This has left due to of compatilibity with the rust abstraction
  public fun remove_trusted_property(
    self : &mut Federation,
    cap : &RootAuthorityCap,
    property_name : TrustedPropertyName,
    _ctx : &mut TxContext)
  {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    self.governance.trusted_constraints.data_mut().remove(&property_name);
  }

  /// Creates a new accredit capability
  fun new_cap_accredit(self : &Federation, ctx : &mut TxContext) : AccreditCap {
    AccreditCap {
      id : object::new(ctx),
      federation_id : self.federation_id(),
    }
  }

  // Revoke trusted property by setting the validity to a specific time
  public fun revoke_trusted_property(federation : &mut Federation, cap : &RootAuthorityCap, property_name : TrustedPropertyName, valid_to_ms : u64) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);
    let property_constraint = federation.governance.trusted_constraints.data_mut().get_mut(&property_name);
    property_constraint.revoke_constraint(valid_to_ms);
  }

  /// Creates a new attest capability
  fun new_cap_attest(self : &Federation, ctx : &mut TxContext) : AttestCap {
    AttestCap {
      id : object::new(ctx),
      federation_id : self.federation_id(),
    }
  }

  public fun add_root_authority(
      self : &mut Federation,
      cap : &RootAuthorityCap,
      account_id : ID,
      ctx : &mut TxContext,
    ) {
    if  (cap.federation_id != self.federation_id()) {
      abort EUnauthorizedWrongFederation
    };
    let root_authority = Self::new_root_authority(account_id, ctx);
    vector::push_back(&mut self.root_authorities, root_authority);

    let cap = Self::new_root_authority_cap(self, ctx);
    transfer::transfer(cap, account_id.to_address());
  }

  fun new_root_authority_cap(self : &Federation, ctx : &mut TxContext )  : RootAuthorityCap {
    RootAuthorityCap {
      id : object::new(ctx),
      federation_id: self.federation_id()
    }
  }

  public(package) fun new_root_authority(account_id: ID, ctx: &mut TxContext)  : RootAuthority {
    RootAuthority {
      id : object::new(ctx),
      account_id : account_id,
    }
  }


  /// Issue an accredidation to accredit about given trusted properties
  public fun create_accreditation(self : &mut Federation, cap : &AccreditCap,  receiver : ID, want_property_constraints : vector<TrustedPropertyConstraint>,  ctx : &mut TxContext) {
      assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
      let current_time_ms = ctx.epoch_timestamp_ms();

      // Check the permissions only if the sender is not a root authority
      if (! self.is_root_authority(&ctx.sender().to_id())) {
        let permissions_to_accredit = self.get_accreditations(&ctx.sender().to_id());
        assert!(permissions_to_accredit.are_constraints_permitted(&want_property_constraints, current_time_ms), EUnauthorizedInsufficientAccreditation);
      };

      let mut trusted_constraints :VecMap<TrustedPropertyName, TrustedPropertyConstraint> =  vec_map::empty();
      let want_property_constraints_len = vector::length<TrustedPropertyConstraint>(&want_property_constraints);
      let mut idx = 0;
      while (idx < want_property_constraints_len ) {
        trusted_constraints.insert(*want_property_constraints[idx].property_name(), want_property_constraints[idx]);
        idx = idx + 1;
      };


      let permission = permission::new_permission(self.federation_id(), trusted_constraints, ctx);
      if ( self.governance.accreditors.contains(&receiver) ) {
          self.governance.accreditors.get_mut(&receiver).add(permission);
        } else {
          let mut permissions_to_accredit  = permission::new_permissions();
          permissions_to_accredit.add(permission);
          self.governance.accreditors.insert(receiver, permissions_to_accredit);

          // also create a capability
          transfer::transfer(self.new_cap_accredit(ctx), receiver.to_address());
        }
  }

  /// creates a permission  (permission_to_attest) to attest about attributes
  public fun create_attestation(self : &mut Federation, cap : &AttestCap, receiver : ID, wanted_constraints: vector<TrustedPropertyConstraint>, ctx : &mut TxContext) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let current_time_ms = ctx.epoch_timestamp_ms();

    // Check the permissions only if the sender is not a root authority
    if (! self.is_root_authority(&ctx.sender().to_id())) {
      let permissions_to_accredit = self.get_accreditations(&ctx.sender().to_id());
      assert!(permissions_to_accredit.are_constraints_permitted(&wanted_constraints, current_time_ms), EUnauthorizedInsufficientAccreditation);
    };

    let permission = permission::new_permission(
      self.federation_id(), trusted_constraint::to_map_of_constraints(wanted_constraints), ctx
    );

    if ( self.governance.attesters.contains(&receiver))  {
      self.governance.attesters.get_mut(&receiver).add_permission_to_attest(permission);
    } else {
        let mut permissions_to_attest = permission::new_empty_permission();
        permissions_to_attest.add_permission_to_attest(permission);
        self.governance.attesters.insert(receiver, permissions_to_attest);

        // also create a capability
        transfer::transfer(self.new_cap_attest(ctx), receiver.to_address());
    };
  }

  public fun revoke_attestation(self : &mut Federation, cap : &AttestCap, user_id : &ID,  permission_id : &ID, ctx : &mut TxContext) {
    let current_time_ms = ctx.epoch_timestamp_ms();
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let remover_permissions = self.get_attestations(&ctx.sender().to_id());

    let users_attest_permissions = self.get_attestations(user_id);
    let mut permission_to_revoke_idx = users_attest_permissions.find_permission_idx(permission_id);
    assert!(permission_to_revoke_idx.is_some(), EPermissionNotFound);

    // Make suere that the sender has the right to revoke the permission
    if (! self.is_root_authority(&ctx.sender().to_id())) {
      let permission_to_revoke = &users_attest_permissions.permisssions()[permission_to_revoke_idx.extract()];
      let (_, constraints) = (*permission_to_revoke.constraints()).into_keys_values() ;
      assert!(remover_permissions.are_constraints_permitted(&constraints, current_time_ms), EUnauthorizedInsufficientAttestation);
    };

    // Remove the permission
    let users_attest_permissions =  self.governance.attesters.get_mut(user_id);
    users_attest_permissions.remove_permission(permission_id);
  }


  public fun revoke_accreditation(self : &mut Federation, cap : &AccreditCap, user_id : &ID,  permission_id : &ID, ctx : &mut TxContext) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    let remover_permissions = self.get_accreditations(&ctx.sender().to_id());

    let users_accredit_permissions = self.get_accreditations(user_id);
    let mut permission_to_revoke_idx = users_accredit_permissions.find_permission_idx(permission_id);
    assert!(permission_to_revoke_idx.is_some(), EPermissionNotFound);

    // make suere that the sender has the right to revoke the permission
    if (! self.is_root_authority(&ctx.sender().to_id())) {
      let permission_to_revoke = &users_accredit_permissions.permisssions()[permission_to_revoke_idx.extract()];
      let current_time_ms   = ctx.epoch_timestamp_ms();
      let (_, constraints) = (*permission_to_revoke.constraints()).into_keys_values() ;
      assert!(remover_permissions.are_constraints_permitted(&constraints, current_time_ms), EUnauthorizedInsufficientAccreditation);
    };

    // Remove the permission
    let users_accredit_permissions =  self.governance.accreditors.get_mut(user_id);
    users_accredit_permissions.remove_permission(permission_id);
  }

  public fun validate_trusted_properties(self : &Federation, issuer_id : &ID, trusted_properties: VecMap<TrustedPropertyName, TrustedPropertyValue>, ctx : &mut TxContext) {
    // ? Should the user of trust framework be allowed to use any timestamp to validate the properites at any point in time?
    let current_time_ms = ctx.epoch_timestamp_ms();
    let property_names = trusted_properties.keys();

    let mut idx = 0;
    while (idx < property_names.length()) {
      let property_name = property_names[idx];
      assert!(
        self.is_trusted_property(property_name),
        EInvalidProperty,
      );
      idx = idx + 1;
    };
    // then check if names and values are permitted for given issuer
    let issuer_permissions_to_attest = self.get_attestations(issuer_id);
    assert!(
      issuer_permissions_to_attest.are_values_permitted(&trusted_properties, current_time_ms),
      EInvalidIssuerInsufficientAttestation,
    );
  }

  public(package) fun root_authorities(self : &Federation) : &vector<RootAuthority> {
    &self.root_authorities
  }

}


#[test_only]
module ith::main_tests {
  use std::string::{utf8};
  use ith::main::{
    new_federation, RootAuthorityCap, Federation, AccreditCap,AttestCap,
    add_trusted_property, create_accreditation, create_attestation,
    revoke_attestation, revoke_accreditation,
  };
  use iota::test_scenario;
  use iota::vec_set::{Self};
  use ith::trusted_property::{new_property_value_number, new_property_name};

  #[test]
  fun creating_new_federation_works() {
    let alice = @0x1;

    let mut scenario = test_scenario::begin(alice);

    // create new federation
    new_federation(scenario.ctx());

    scenario.next_tx(alice);

    // Check that the alice has RootAuthorityCap
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // check the federation
    let fed: Federation = scenario.take_shared();

    assert!(fed.is_accreditor(&alice.to_id()), 0);
    assert!(fed.is_attester(&alice.to_id()), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);

    let _ = scenario.end();
  }

  #[test]
  fun test_adding_root_authority_to_the_federation() {
    let alice = @0x1;

    let mut scenario = test_scenario::begin(alice);

    let new_object = scenario.new_object();
    let bob = new_object.uid_to_inner();

    scenario.next_tx(alice);

    // Create a new federation
    new_federation(scenario.ctx());

    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a new root authority
    fed.add_root_authority(&cap, bob, scenario.ctx());

    scenario.next_tx(alice);

    // check that bob has RootAuthorityCap
    let bob_cap: RootAuthorityCap = scenario.take_from_address(bob.to_address());

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(bob.to_address(), bob_cap);
    test_scenario::return_shared(fed);
    new_object.delete();

    let _ = scenario.end();
  }

  #[test]
  fun test_adding_trusted_property() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a trusted property
    let property_name = new_property_name(utf8(b"property_name"));
    let property_value = new_property_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    fed.add_trusted_property(&cap,property_name, allowed_values, false, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the property was added
    assert!(fed.is_trusted_property(property_name), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);

    let _ = scenario.end();
  }

  #[test]
  fun test_create_accreditation() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let constraints = vector::empty();
    fed.create_accreditation(&accredit_cap, bob, constraints, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was issued
    assert!(fed.is_accreditor(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
  }

  #[test]
  fun test_create_attestation() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let attest_cap: AttestCap = scenario.take_from_address(alice);
    // Add a trusted property

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let constraints = vector::empty();
    fed.create_attestation(&attest_cap, bob, constraints, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was issued
    assert!(fed.is_attester(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
  }

  #[test]
  fun test_revoke_attestation_and_accredit() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);
    let attest_cap: AttestCap = scenario.take_from_address(alice);

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let constraints = vector::empty();
    fed.create_accreditation(&accredit_cap, bob, constraints, scenario.ctx());
    scenario.next_tx(alice);

    // Issue permission to attest
    fed.create_attestation(&attest_cap, bob, constraints, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to attest
    let permission_id = fed.get_attestations(&bob).permisssions()[0].id().uid_to_inner();
    fed.revoke_attestation(&attest_cap, &bob, &permission_id, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to accredit
    let permission_id = fed.get_accreditations(&bob).permisssions()[0].id().uid_to_inner();
    fed.revoke_accreditation(&accredit_cap, &bob, &permission_id, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was revoked
    // TODO::@itsyaasir: This should be fixed since the user has no permissions
    // and should not be able to attest/accredit
    assert!(fed.is_attester(&bob), 0);
    assert!(fed.is_accreditor(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
  }

}
