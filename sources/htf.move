// HTF Notary module
module htf::main {
  use iota::vec_map::{Self, VecMap};
  use iota::event;
  use iota::vec_set::{VecSet};

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
    account_id: ID,
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
      root_authorities : vector::empty(),
      governance : Governance {
        id : object::new(ctx),
        trusted_constraints: trusted_constraint::new_trusted_property_constraints(),
        accreditors : vec_map::empty(),
        attesters : vec_map::empty(),
        credentials_state : vec_map::empty(),
      },
    };

    let root_auth_cap = Self::new_root_authority_cap(&federation, ctx);
    let root_authority = Self::new_root_authority(ctx.sender().to_id(), ctx);
    vector::push_back(&mut federation.root_authorities, root_authority);

    // Add permission to attest
    let permission = permission_to_accredit::new_permissions_to_accredit();
    federation.governance.accreditors.insert(ctx.sender().to_id(), permission);

    // Add permission to attest
    let permission = permission_to_attest::new_permissions_to_attest();
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

  public fun get_federation_properties(self : &Federation) : vector<TrustedPropertyName> {
    self.governance.trusted_constraints.data().keys()
  }

  public fun has_federation_property(self : &Federation, property_name : TrustedPropertyName) : bool {
    self.governance.trusted_constraints.data().contains(&property_name)
  }

  public(package) fun find_permissions_to_attest(self: &Federation, user_id : &ID)  :  &PermissionsToAttest {
      self.governance.attesters.get(user_id)
  }


  public fun has_permissions_to_attest(self : &Federation, user_id : &ID)  : bool {
    self.governance.attesters.contains(user_id)
  }

  public(package) fun find_permissions_to_accredit(self : &Federation, user_id : &ID) : &PermissionsToAccredit {
    self.governance.accreditors.get(user_id)
  }


  public fun has_permissions_to_accredit(self : &Federation, user_id :&ID)  : bool {
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
    );

    self.governance.trusted_constraints.add_constraint(property_name, constraint) ;
  }

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
  public fun issue_permission_to_accredit(self : &mut Federation, cap : &AccreditCap,  receiver : ID, want_property_constraints : vector<TrustedPropertyConstraint>,  ctx : &mut TxContext) {
      assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

      let permissions_to_accredit = self.find_permissions_to_accredit(&ctx.sender().to_id());
      assert!(permissions_to_accredit.are_constraints_permitted(&want_property_constraints), EUnauthorizedInsufficientAccreditation);

      let mut trusted_constraints :VecMap<TrustedPropertyName, TrustedPropertyConstraint> =  vec_map::empty();
      let want_property_constraints_len = vector::length<TrustedPropertyConstraint>(&want_property_constraints);
      let mut idx = 0;
      while (idx < want_property_constraints_len ) {
        trusted_constraints.insert(*want_property_constraints[idx].property_name(), want_property_constraints[idx]);
        idx = idx + 1;
      };


      let permission = permission_to_accredit::new_permission_to_accredit(self.federation_id(), trusted_constraints, ctx);
      if ( self.governance.accreditors.contains(&receiver) ) {
          self.governance.accreditors.get_mut(&receiver).add(permission);
        } else {
          let mut permissions_to_accredit  = permission_to_accredit::new_permissions_to_accredit();
          permissions_to_accredit.add(permission);
          self.governance.accreditors.insert(receiver, permissions_to_accredit);

          // also create a capability
          transfer::transfer(self.new_cap_accredit(ctx), receiver.to_address());
        }
  }

  /// creates a permission  (permission_to_attest) to attest about attributes
  public fun issue_permission_to_attest(self : &mut Federation, cap : &AttestCap, receiver : ID, wanted_constraints: vector<TrustedPropertyConstraint>, ctx : &mut TxContext) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let permissions_to_accredit = self.find_permissions_to_accredit(&ctx.sender().to_id());
    assert!(permissions_to_accredit.are_constraints_permitted(&wanted_constraints), EUnauthorizedInsufficientAccreditation);

    let permission = permission_to_attest::new_permission_to_attest(
      self.federation_id(), trusted_constraint::to_map_of_constraints(wanted_constraints), ctx
    );

    if ( self.governance.attesters.contains(&receiver))  {
      self.governance.attesters.get_mut(&receiver).add_permission_to_attest(permission);
    } else {
        let mut permissions_to_attest = permission_to_attest::new_permissions_to_attest();
        permissions_to_attest.add_permission_to_attest(permission);
        self.governance.attesters.insert(receiver, permissions_to_attest);

        // also create a capability
        transfer::transfer(self.new_cap_attest(ctx), receiver.to_address());
    };
  }

  public fun revoke_permission_to_attest(self : &mut Federation, cap : &AttestCap, user_id : &ID,  permission_id : &ID, ctx : &mut TxContext) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let remover_permissions = self.find_permissions_to_attest(&ctx.sender().to_id());

    let users_attest_permissions = self.find_permissions_to_attest(user_id);
    let mut permission_to_revoke_idx = users_attest_permissions.find_permission_idx(permission_id);
    assert!(permission_to_revoke_idx.is_some(), EPermissionNotFound);

    // Make suere that the sender has the right to revoke the permission
    let permission_to_revoke = &users_attest_permissions.permisssions()[permission_to_revoke_idx.extract()];
    let (_, constraints) = (*permission_to_revoke.constraints()).into_keys_values() ;
    assert!(remover_permissions.are_constraints_permitted(&constraints), EUnauthorizedInsufficientAttestation);

    // Remove the permission
    let users_attest_permissions =  self.governance.attesters.get_mut(user_id);
    users_attest_permissions.remove_permission(permission_id);
  }


  public fun revoke_permission_to_accredit(self : &mut Federation, cap : &AccreditCap, user_id : &ID,  permission_id : &ID, ctx : &mut TxContext) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let remover_permissions = self.find_permissions_to_accredit(&ctx.sender().to_id());

    let users_accredit_permissions = self.find_permissions_to_accredit(user_id);
    let mut permission_to_revoke_idx = users_accredit_permissions.find_permission_idx(permission_id);
    assert!(permission_to_revoke_idx.is_some(), EPermissionNotFound);

    // make suere that the sender has the right to revoke the permission
    let permission_to_revoke = &users_accredit_permissions.permisssions()[permission_to_revoke_idx.extract()];
    let (_, constraints) = (*permission_to_revoke.constriants()).into_keys_values() ;
    assert!(remover_permissions.are_constraints_permitted(&constraints), EUnauthorizedInsufficientAccreditation);

    // Remove the permission
    let users_accredit_permissions =  self.governance.accreditors.get_mut(user_id);
    users_accredit_permissions.remove_permission(permission_id);
  }

  public fun issue_credential(
      self : &mut Federation,
      cap : &AttestCap,
      receiver : ID,
      trusted_properties : VecMap<TrustedPropertyName, TrustedPropertyValue>,
      valid_from_ts_ms : u64,
      valid_until_ts_ms : u64,
      ctx : &mut TxContext)  {
      assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

      let permissions_to_attest = self.find_permissions_to_attest(&ctx.sender().to_id());
      assert!(permissions_to_attest.are_values_permitted(&trusted_properties), EUnauthorizedInsufficientAttestation);

      let creds = new_credential(trusted_properties, valid_from_ts_ms, valid_until_ts_ms, receiver, ctx);
      self.governance.credentials_state.insert(creds.id.to_inner(), new_credential_state());

      transfer::transfer(creds, receiver.to_address());
  }

  public fun validate_trusted_properties(self : &Federation, issuer_id : &ID, trusted_properties: VecMap<TrustedPropertyName, TrustedPropertyValue>) {
    // check if every property belongs to the federation
    let property_names = trusted_properties.keys();
    let mut idx = 0;
    while (idx < property_names.length()) {
      let property_name = property_names[idx];
      assert!(
        self.has_federation_property(property_name),
        EInvalidProperty,
      );
      idx = idx + 1;
    };
    // then check if names and values are permitted for given issuer
    let issuer_permissions_to_attest = self.find_permissions_to_attest(issuer_id);
    assert!(
      issuer_permissions_to_attest.are_values_permitted(&trusted_properties),
      EInvalidIssuerInsufficientAttestation,
    );
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

  public(package) fun root_authorities(self : &Federation) : &vector<RootAuthority> {
    &self.root_authorities
  }

}


#[test_only]
module htf::main_tests {
  use std::string::{utf8};
  use htf::main::{
    new_federation, RootAuthorityCap, Federation, AccreditCap,AttestCap, Credential,
    add_trusted_property, issue_permission_to_accredit, issue_permission_to_attest,
    revoke_permission_to_attest, revoke_permission_to_accredit, issue_credential
  };
  use iota::test_scenario;
  use iota::vec_set::{Self};
  use iota::vec_map;
  use htf::trusted_property::{new_property_value_number, new_property_name};
  use htf::trusted_constraint::{new_trusted_property_constraint};

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

    assert!(fed.has_permissions_to_accredit(&alice.to_id()), 0);
    assert!(fed.has_permissions_to_attest(&alice.to_id()), 0);

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
    assert!(fed.has_federation_property(property_name), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);

    let _ = scenario.end();
  }

  #[test]
  fun test_issue_permission_to_accredit() {
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
    fed.issue_permission_to_accredit(&accredit_cap, bob, constraints, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was issued
    assert!(fed.has_permissions_to_accredit(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
  }

  #[test]
  fun test_issue_permission_to_attest() {
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
    fed.issue_permission_to_attest(&attest_cap, bob, constraints, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was issued
    assert!(fed.has_permissions_to_attest(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
  }

  #[test]
  fun test_revoke_permission_to_attest_and_accredit() {
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
    fed.issue_permission_to_accredit(&accredit_cap, bob, constraints, scenario.ctx());
    scenario.next_tx(alice);

    // Issue permission to attest
    fed.issue_permission_to_attest(&attest_cap, bob, constraints, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to attest
    let permission_id = fed.find_permissions_to_attest(&bob).permisssions()[0].id().uid_to_inner();
    fed.revoke_permission_to_attest(&attest_cap, &bob, &permission_id, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to accredit
    let permission_id = fed.find_permissions_to_accredit(&bob).permisssions()[0].id().uid_to_inner();
    fed.revoke_permission_to_accredit(&accredit_cap, &bob, &permission_id, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the permission was revoked
    // TODO::@itsyaasir: This should be fixed since the user has no permissions
    // and should not be able to attest/accredit
    assert!(fed.has_permissions_to_attest(&bob), 0);
    assert!(fed.has_permissions_to_accredit(&bob), 0);

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_shared(fed);

    new_id.delete();

    let _ = scenario.end();
  }

  #[test]
  fun test_issue_credential() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);
    let attest_cap: AttestCap = scenario.take_from_address(alice);
    let accredit_cap: AccreditCap = scenario.take_from_address(alice);

    // Add a trusted property
    let property_name = new_property_name(utf8(b"property_name"));
    let property_value = new_property_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    fed.add_trusted_property(&cap, property_name, allowed_values, false, scenario.ctx());
    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    let mut wanted_constraints = vector::empty();
    let constraints = new_trusted_property_constraint(
      property_name, allowed_values, true
    );

    wanted_constraints.push_back(constraints);

    fed.issue_permission_to_attest(&attest_cap, bob, wanted_constraints, scenario.ctx());
    scenario.next_tx(bob.to_address());

    // Issue credential
    let mut trusted_properties = vec_map::empty();
    trusted_properties.insert(property_name, property_value);
    fed.issue_credential(&attest_cap, bob, trusted_properties, 0, 1000, scenario.ctx());

    scenario.next_tx(alice);

    // Let us validate the credential
    let cred: Credential = scenario.take_from_address(bob.to_address());

    // Validate the credential
    fed.validate_credential(&cred, scenario.ctx());

    // Return the cap to the alice
    test_scenario::return_to_address(alice, cap);
    test_scenario::return_shared(fed);
    test_scenario::return_to_address(alice, accredit_cap);
    test_scenario::return_to_address(alice, attest_cap);
    test_scenario::return_to_address(bob.to_address(), cred);


    new_id.delete();
    let _ = scenario.end();
  }
}
