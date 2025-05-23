// HTF Notary module
module ith::main {
  use iota::vec_map::{Self, VecMap};
  use iota::event;
  use iota::vec_set::{VecSet};

  use ith::statement_name::{StatementName};
  use ith::statement_value::{StatementValue};
  use ith::statement::{Self, Statements, Statement};
  use ith::accreditation::{Self, Accreditations};

  const  EUnauthorizedWrongFederation  : u64  = 1;
  const  EUnauthorizedInsufficientAccreditation : u64 = 2;
  const  EUnauthorizedInsufficientAttestation : u64 = 3;
  const  EInvalidProperty: u64 = 4;
  const  EInvalidIssuerInsufficientAccreditation: u64 = 5;
  const  EInvalidConstraint  : u64 = 6;
  const  EAccreditationNotFound: u64 = 7;

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


  // Governance is the object that contains the Statements and tracks the abilities of entities to attest and accredit
  public struct Governance has store, key {
    id : UID,
    /// Statements that are trusted by the given Federation
    statements : Statements,
    /// Accreditation rights to delegate accreditation permissions.
    /// These accreditations allow a user to grant other users the ability to further delegate accreditation permissions.
    accreditations_to_accredit : VecMap<ID, Accreditations>,
    /// Accreditation rights for attestation.
    /// These accreditations empower a user to create Attestations.
    accreditations_to_attest : VecMap<ID, Accreditations>,
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
        statements: statement::new_statements(),
        accreditations_to_accredit : vec_map::empty(),
        accreditations_to_attest : vec_map::empty(),
      },
    };

    let root_auth_cap = Self::new_root_authority_cap(&federation, ctx);
    let root_authority = Self::new_root_authority(ctx.sender().to_id(), ctx);
    vector::push_back(&mut federation.root_authorities, root_authority);

    // Add permission to attest
    let permission = accreditation::new_empty_accreditations();
    federation.governance.accreditations_to_accredit.insert(ctx.sender().to_id(), permission);

    // Add permission to attest
    let permission = accreditation::new_empty_accreditations();
    federation.governance.accreditations_to_attest.insert(ctx.sender().to_id(), permission);

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

  public fun get_statements(self : &Federation) : vector<StatementName> {
    self.governance.statements.data().keys()
  }

  public fun is_statement_in_federation(self : &Federation, statement_name : StatementName) : bool {
    self.governance.statements.data().contains(&statement_name)
  }

  public fun get_accreditations_to_attest(self: &Federation, user_id : &ID)  :  &Accreditations {
      self.governance.accreditations_to_attest.get(user_id)
  }

  public fun is_attester(self : &Federation, user_id : &ID)  : bool {
    self.governance.accreditations_to_attest.contains(user_id)
  }

  public fun get_accreditations_to_accredit(self : &Federation, user_id : &ID) : &Accreditations {
    self.governance.accreditations_to_accredit.get(user_id)
  }

  public fun is_accreditor(self : &Federation, user_id :&ID)  : bool {
    self.governance.accreditations_to_accredit.contains(user_id)
  }

  public fun add_statement(
    self : &mut Federation,
    cap : &RootAuthorityCap,
    statement_name : StatementName,
    allowed_values : VecSet<StatementValue>,
    allow_any : bool,
    _ctx : &mut TxContext)
  {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    assert!(!(allow_any && allowed_values.keys().length() > 0), EInvalidConstraint);

    let statement = statement::new_statement(
      statement_name,
      allowed_values,
      allow_any,
      option::none(),
    );

    self.governance.statements.add_statement(statement) ;
  }


  // TODO should be removed
  // This has left due to of compatilibity with the rust abstraction
  public fun remove_statement(
    self : &mut Federation,
    cap : &RootAuthorityCap,
    statement_name : StatementName,
    _ctx : &mut TxContext)
  {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    self.governance.statements.data_mut().remove(&statement_name);
  }

  /// Creates a new accredit capability
  fun new_cap_accredit(self : &Federation, ctx : &mut TxContext) : AccreditCap {
    AccreditCap {
      id : object::new(ctx),
      federation_id : self.federation_id(),
    }
  }

  // Revoke Statement by setting the validity to a specific time
  public fun revoke_trusted_statement(federation : &mut Federation, cap : &RootAuthorityCap, statement_name : StatementName, valid_to_ms : u64) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);
    let property_statement = federation.governance.statements.data_mut().get_mut(&statement_name);
    property_statement.revoke(valid_to_ms);
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


  // Accredit is a process of transferring trust to another entity. It allows the entity to accredit other entities.
  public fun accredit(self : &mut Federation, cap : &AccreditCap,  receiver : ID, want_statements : vector<Statement>,  ctx : &mut TxContext) {
      assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
      let current_time_ms = ctx.epoch_timestamp_ms();

      // Check the permissions only if the sender is not a root authority
      if (! self.is_root_authority(&ctx.sender().to_id())) {
        let accreditations_to_accredit = self.get_accreditations_to_accredit(&ctx.sender().to_id());
        assert!(accreditations_to_accredit.are_statements_compliant(&want_statements, current_time_ms), EUnauthorizedInsufficientAccreditation);
      };

      let accreditation = accreditation::new_accreditation(want_statements, ctx);
      if ( self.governance.accreditations_to_accredit.contains(&receiver) ) {
          self.governance.accreditations_to_accredit.get_mut(&receiver).add_accreditation(accreditation);
        } else {
          let mut accreditations  = accreditation::new_empty_accreditations();
          accreditations.add_accreditation(accreditation);
          self.governance.accreditations_to_accredit.insert(receiver, accreditations);

          // also create a capability
          transfer::transfer(self.new_cap_accredit(ctx), receiver.to_address());
        }
  }

  /// Accredit to attest is a a process when trusted entity can make make another entity the Attester. Attester can make statements that are trusted by the federation.
  public fun accredit_to_attest(self : &mut Federation, cap : &AttestCap, receiver : ID, wanted_statements: vector<Statement>, ctx : &mut TxContext) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let current_time_ms = ctx.epoch_timestamp_ms();

    // Check the permissions only if the sender is not a root authority
    if (! self.is_root_authority(&ctx.sender().to_id())) {
      let accreditations_to_accredit = self.get_accreditations_to_accredit(&ctx.sender().to_id());
      assert!(accreditations_to_accredit.are_statements_compliant(&wanted_statements, current_time_ms), EUnauthorizedInsufficientAccreditation);
    };

    let accredited_statement = accreditation::new_accreditation(wanted_statements, ctx);

    if ( self.governance.accreditations_to_attest.contains(&receiver))  {
      self.governance.accreditations_to_attest.get_mut(&receiver).add_accreditation(accredited_statement);
    } else {
        let mut accreditations_to_attest = accreditation::new_empty_accreditations();
        accreditations_to_attest.add_accreditation(accredited_statement);
        self.governance.accreditations_to_attest.insert(receiver, accreditations_to_attest);

        // also create a capability
        transfer::transfer(self.new_cap_attest(ctx), receiver.to_address());
    };
  }

  public fun revoke_accreditation_to_attest(self : &mut Federation, cap : &AttestCap, user_id : &ID,  permission_id : &ID, ctx : &mut TxContext) {
    let current_time_ms = ctx.epoch_timestamp_ms();
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let remover_accreditations = self.get_accreditations_to_attest(&ctx.sender().to_id());

    let users_attest_permissions = self.get_accreditations_to_attest(user_id);
    let mut accreditation_to_revoke_idx = users_attest_permissions.find_accredited_statement_id(permission_id);
    assert!(accreditation_to_revoke_idx.is_some(), EAccreditationNotFound);

    // Make sure that the sender has the right to revoke the permission
    if (! self.is_root_authority(&ctx.sender().to_id())) {
      let accreditation_to_revoke = &users_attest_permissions.accredited_statements()[accreditation_to_revoke_idx.extract()];
      let (_, statements) = (*accreditation_to_revoke.statements()).into_keys_values() ;
      assert!(remover_accreditations.are_statements_compliant(&statements, current_time_ms), EUnauthorizedInsufficientAttestation);
    };

    // Remove the permission
    let users_attest_permissions =  self.governance.accreditations_to_attest.get_mut(user_id);
    users_attest_permissions.remove_accredited_statement(permission_id);
  }


  public fun revoke_accreditation_to_accredit(self : &mut Federation, cap : &AccreditCap, user_id : &ID,  permission_id : &ID, ctx : &mut TxContext) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    let remover_permissions = self.get_accreditations_to_accredit(&ctx.sender().to_id());

    let users_accredit_permissions = self.get_accreditations_to_accredit(user_id);
    let mut accreditation_to_revoke_idx = users_accredit_permissions.find_accredited_statement_id(permission_id);
    assert!(accreditation_to_revoke_idx.is_some(), EAccreditationNotFound);

    // Make sure that the sender has the right to revoke the permission
    if (! self.is_root_authority(&ctx.sender().to_id())) {
      let accreditation_to_revoke = &users_accredit_permissions.accredited_statements()[accreditation_to_revoke_idx.extract()];
      let current_time_ms   = ctx.epoch_timestamp_ms();
      let (_, statements) = (*accreditation_to_revoke.statements()).into_keys_values() ;
      assert!(remover_permissions.are_statements_compliant(&statements, current_time_ms), EUnauthorizedInsufficientAccreditation);
    };

    // Remove the permission
    let users_accredit_permissions =  self.governance.accreditations_to_accredit.get_mut(user_id);
    users_accredit_permissions.remove_accredited_statement(permission_id);
  }

  public fun validate_statement(self : &Federation, attester_id : &ID, statement_name : StatementName, statement_value : StatementValue, ctx : &mut TxContext) {
    let current_time_ms = ctx.epoch_timestamp_ms();
    assert!(self.is_statement_in_federation(statement_name), EInvalidProperty);

    let accreditations = self.get_accreditations_to_attest(attester_id);
    assert!(
      accreditations.is_statement_allowed(&statement_name, &statement_value, current_time_ms),
      EInvalidIssuerInsufficientAccreditation,
    );
  }

  public fun validate_statements(self : &Federation, issuer_id : &ID, statements: VecMap<StatementName, StatementValue>, ctx : &mut TxContext) {
    let current_time_ms = ctx.epoch_timestamp_ms();
    let statement_names = statements.keys();

    let mut idx = 0;
    while (idx < statement_names.length()) {
      let statement_name = statement_names[idx];
      assert!(
        self.is_statement_in_federation(statement_name),
        EInvalidProperty,
      );
      idx = idx + 1;
    };
    // then check if names and values are permitted for given issuer
    let accreditations = self.get_accreditations_to_attest(issuer_id);
    assert!(
      accreditations.are_statements_allowed(&statements, current_time_ms),
      EInvalidIssuerInsufficientAccreditation,
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
    add_statement, accredit_to_attest, accredit,
    revoke_accreditation_to_attest, revoke_accreditation_to_accredit,
  };
  use iota::test_scenario;
  use iota::vec_set::{Self};
  use ith::statement_name::{new_statement_name};
  use ith::statement_value::{new_property_value_number};

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
  fun test_adding_trusted_statement() {
    let alice = @0x1;
    let mut scenario = test_scenario::begin(alice);

    // Create a new federation
    new_federation(scenario.ctx());
    scenario.next_tx(alice);

    let mut fed: Federation = scenario.take_shared();
    let cap: RootAuthorityCap = scenario.take_from_address(alice);

    // Add a Statement
    let statement_name = new_statement_name(utf8(b"statement_name"));
    let property_value = new_property_value_number(10);
    let mut allowed_values = vec_set::empty();
    allowed_values.insert(property_value);

    fed.add_statement(&cap,statement_name, allowed_values, false, scenario.ctx());
    scenario.next_tx(alice);

    // Check if the property was added
    assert!(fed.is_statement_in_federation(statement_name), 0);

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
    let statements = vector::empty();
    fed.accredit(&accredit_cap, bob, statements, scenario.ctx());
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
    // Add a Statement

    scenario.next_tx(alice);

    let new_id = scenario.new_object();
    let bob = new_id.uid_to_inner();

    // Issue permission to accredit
    let statements = vector::empty();
    fed.accredit_to_attest(&attest_cap, bob, statements, scenario.ctx());
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
  fun test_revoke_accreditation_to_attest_and_accredit() {
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
    let statements = vector::empty();
    fed.accredit(&accredit_cap, bob, statements, scenario.ctx());
    scenario.next_tx(alice);

    // Issue permission to attest
    fed.accredit_to_attest(&attest_cap, bob, statements, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to attest
    let permission_id = fed.get_accreditations_to_attest(&bob).accredited_statements()[0].id().uid_to_inner();
    fed.revoke_accreditation_to_attest(&attest_cap, &bob, &permission_id, scenario.ctx());
    scenario.next_tx(alice);

    // Revoke permission to accredit
    let permission_id = fed.get_accreditations_to_accredit(&bob).accredited_statements()[0].id().uid_to_inner();
    fed.revoke_accreditation_to_accredit(&accredit_cap, &bob, &permission_id, scenario.ctx());
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
