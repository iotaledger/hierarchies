/// This module implements a hierarchical trust system where entities can delegate
/// trust and attestation rights to other entities within a federation.
module ith::main {
    use iota::event;
    use iota::vec_map::{Self, VecMap};
    use iota::vec_set::VecSet;
    use ith::accreditation::{Self, Accreditations};
    use ith::statement::{Self, Statements, Statement};
    use ith::statement_name::StatementName;
    use ith::statement_value::StatementValue;

    // ===== Errors =====
    /// Error when operation is performed with wrong federation
    const EUnauthorizedWrongFederation: u64 = 1;
    /// Error when entity lacks sufficient accreditation permissions
    const EUnauthorizedInsufficientAccreditationToAccredit: u64 = 2;
    /// Error when entity lacks sufficient attestation permissions
    const EUnauthorizedInsufficientAccreditationToAttest: u64 = 3;
    /// Error when property/statement is invalid
    const EInvalidStatement: u64 = 4;
    /// Error when attester has insufficient accreditation for the statement
    const EAttesterInsufficientAccreditation: u64 = 5;
    /// Error when Value Condition for Statement is invalid (e.g., allow_any=true with specific values)
    const EInvalidStatementValueCondition: u64 = 6;
    /// Error when trying to access non-existent accreditation
    const EAccreditationNotFound: u64 = 7;

    // ===== Core Data Structures =====

    /// The main federation object representing a hierarchy of trust.
    /// This is a shared object that maintains the trust structure and governance.
    public struct Federation has key, store {
        id: UID,
        governance: Governance,
        root_authorities: vector<RootAuthority>,
    }

    /// Root authority with the highest trust level in the system.
    /// Can delegate and revoke trust to other entities.
    public struct RootAuthority has key, store {
        id: UID,
        account_id: ID,
    }

    /// Governance structure containing trusted statements and accreditation tracking.
    /// Manages what statements are trusted and who can attest/accredit.
    public struct Governance has key, store {
        id: UID,
        /// Statements that are trusted by the federation
        statements: Statements,
        /// Rights to delegate accreditation
        accreditations_to_accredit: VecMap<ID, Accreditations>,
        /// Rights for creating attestations
        accreditations_to_attest: VecMap<ID, Accreditations>,
    }

    // ===== Capability Objects =====

    /// Capability for root authority operations
    public struct RootAuthorityCap has key {
        id: UID,
        federation_id: ID,
    }

    /// Capability for attestation operations
    public struct AttestCap has key {
        id: UID,
        federation_id: ID,
    }

    /// Capability for accreditation operations
    public struct AccreditCap has key {
        id: UID,
        federation_id: ID,
    }

    // ===== Event Structures =====

    /// Generic event wrapper
    public struct Event<D> has copy, drop {
        data: D,
    }

    /// Event emitted when a new federation is created
    public struct FederationCreatedEvent has copy, drop {
        federation_address: address,
    }

    // ===== Constructor Functions =====

    /// Creates a new federation with the sender as the first root authority.
    /// The creator of the Federation becomes the root authority of the Federation.
    public fun new_federation(ctx: &mut TxContext) {
        let federation_id = object::new(ctx);
        let mut federation = Federation {
            id: federation_id,
            root_authorities: vector::empty(),
            governance: Governance {
                id: object::new(ctx),
                statements: statement::new_statements(),
                accreditations_to_accredit: vec_map::empty(),
                accreditations_to_attest: vec_map::empty(),
            },
        };

        // Create root authority and capabilities
        let root_auth_cap = new_root_authority_cap(&federation, ctx);
        let root_authority = new_root_authority(ctx.sender().to_id(), ctx);
        vector::push_back(&mut federation.root_authorities, root_authority);

        // Grant permissions to the creator
        let permission = accreditation::new_empty_accreditations();
        federation.governance.accreditations_to_accredit.insert(ctx.sender().to_id(), permission);

        let permission = accreditation::new_empty_accreditations();
        federation.governance.accreditations_to_attest.insert(ctx.sender().to_id(), permission);

        // Create and transfer capabilities
        let attest_cap = new_cap_attest(&federation, ctx);
        let accredit_cap = new_cap_accredit(&federation, ctx);

        // Emit federation created event
        event::emit(Event {
            data: FederationCreatedEvent {
                federation_address: federation.federation_id().to_address(),
            },
        });

        // Transfer capabilities to creator
        transfer::transfer(root_auth_cap, ctx.sender());
        transfer::transfer(accredit_cap, ctx.sender());
        transfer::transfer(attest_cap, ctx.sender());

        // Share the federation object
        transfer::share_object(federation)
    }

    /// Creates a new root authority object
    public(package) fun new_root_authority(account_id: ID, ctx: &mut TxContext): RootAuthority {
        RootAuthority {
            id: object::new(ctx),
            account_id: account_id,
        }
    }

    /// Creates a new root authority capability
    fun new_root_authority_cap(self: &Federation, ctx: &mut TxContext): RootAuthorityCap {
        RootAuthorityCap {
            id: object::new(ctx),
            federation_id: self.federation_id(),
        }
    }

    /// Creates a new accreditation capability
    fun new_cap_accredit(self: &Federation, ctx: &mut TxContext): AccreditCap {
        AccreditCap {
            id: object::new(ctx),
            federation_id: self.federation_id(),
        }
    }

    /// Creates a new attestation capability
    fun new_cap_attest(self: &Federation, ctx: &mut TxContext): AttestCap {
        AttestCap {
            id: object::new(ctx),
            federation_id: self.federation_id(),
        }
    }

    // ===== Read Functions =====

    /// Returns the federation's unique identifier
    fun federation_id(self: &Federation): ID {
        self.id.to_inner()
    }

    /// Gets all statement names trusted by the federation
    public fun get_statements(self: &Federation): vector<StatementName> {
        self.governance.statements.data().keys()
    }

    /// Checks if a statement is trusted by the federation
    public fun is_statement_in_federation(self: &Federation, statement_name: StatementName): bool {
        self.governance.statements.data().contains(&statement_name)
    }

    /// Gets accreditations for attestation for a specific entity
    public fun get_accreditations_to_attest(self: &Federation, entity_id: &ID): &Accreditations {
        self.governance.accreditations_to_attest.get(entity_id)
    }

    /// Checks if an entity can create attestations
    public fun is_attester(self: &Federation, entity_id: &ID): bool {
        self.governance.accreditations_to_attest.contains(entity_id)
    }

    /// Gets accreditations for delegation for a specific entity
    public fun get_accreditations_to_accredit(self: &Federation, entity_id: &ID): &Accreditations {
        self.governance.accreditations_to_accredit.get(entity_id)
    }

    /// Checks if an entity can delegate accreditations
    public fun is_accreditor(self: &Federation, entity_id: &ID): bool {
        self.governance.accreditations_to_accredit.contains(entity_id)
    }

    /// Gets the list of root authorities (package-only access)
    public(package) fun root_authorities(self: &Federation): &vector<RootAuthority> {
        &self.root_authorities
    }

    // ===== Write Functions =====

    /// Adds a new trusted statement to the federation.
    /// Only root authorities can perform this operation.
    public fun add_statement(
        self: &mut Federation,
        cap: &RootAuthorityCap,
        statement_name: StatementName,
        allowed_values: VecSet<StatementValue>,
        allow_any: bool,
        _ctx: &mut TxContext,
    ) {
        assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
        assert!(!(allow_any && allowed_values.keys().length() > 0), EInvalidStatementValueCondition);

        let statement = statement::new_statement(
            statement_name,
            allowed_values,
            allow_any,
            option::none(),
        );

        self.governance.statements.add_statement(statement);
    }

    /// Removes a statement from the federation.
    /// TODO: Should be deprecated in favor of revoking statements
    public fun remove_statement(
        self: &mut Federation,
        cap: &RootAuthorityCap,
        statement_name: StatementName,
        _ctx: &mut TxContext,
    ) {
        assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
        self.governance.statements.data_mut().remove(&statement_name);
    }

    /// Revokes a statement by setting its validity period
    public fun revoke_statement(
        federation: &mut Federation,
        cap: &RootAuthorityCap,
        statement_name: StatementName,
        valid_to_ms: u64,
    ) {
        assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);
        let statement = federation
            .governance
            .statements
            .data_mut()
            .get_mut(&statement_name);
        statement.revoke(valid_to_ms);
    }

    /// Adds a new root authority to the federation.
    /// Only existing root authorities can perform this operation.
    public fun add_root_authority(
        self: &mut Federation,
        cap: &RootAuthorityCap,
        account_id: ID,
        ctx: &mut TxContext,
    ) {
        if (cap.federation_id != self.federation_id()) {
            abort EUnauthorizedWrongFederation
        };

        let root_authority = new_root_authority(account_id, ctx);
        vector::push_back(&mut self.root_authorities, root_authority);

        let cap = new_root_authority_cap(self, ctx);
        transfer::transfer(cap, account_id.to_address());
    }

    /// Grants accreditation rights to another entity.
    /// Allows the receiver to delegate accreditation permissions to others.
    public fun create_accreditation_to_accredit(
        self: &mut Federation,
        cap: &AccreditCap,
        receiver: ID,
        want_statements: vector<Statement>,
        ctx: &mut TxContext,
    ) {
        assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
        let current_time_ms = ctx.epoch_timestamp_ms();

        // Check permissions only if sender is not a root authority
        if (!self.is_root_authority(&ctx.sender().to_id())) {
            let accreditations_to_accredit = self.get_accreditations_to_accredit(
                &ctx.sender().to_id(),
            );
            assert!(
                accreditations_to_accredit.are_statements_compliant(
                    &want_statements,
                    current_time_ms,
                ),
                EUnauthorizedInsufficientAccreditationToAccredit,
            );
        };

        let accreditation = accreditation::new_accreditation(want_statements, ctx);
        if (self.governance.accreditations_to_accredit.contains(&receiver)) {
            self
                .governance
                .accreditations_to_accredit
                .get_mut(&receiver)
                .add_accreditation(accreditation);
        } else {
            let mut accreditations = accreditation::new_empty_accreditations();
            accreditations.add_accreditation(accreditation);
            self.governance.accreditations_to_accredit.insert(receiver, accreditations);

            // Create and transfer capability
            transfer::transfer(self.new_cap_accredit(ctx), receiver.to_address());
        }
    }

    /// Grants attestation rights to another entity.
    /// Allows the receiver to create trusted attestations.
    public fun create_accreditation_to_attest(
        self: &mut Federation,
        cap: &AttestCap,
        receiver: ID,
        wanted_statements: vector<Statement>,
        ctx: &mut TxContext,
    ) {
        assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
        let current_time_ms = ctx.epoch_timestamp_ms();

        // Check permissions only if sender is not a root authority
        if (!self.is_root_authority(&ctx.sender().to_id())) {
            let accreditations_to_accredit = self.get_accreditations_to_accredit(
                &ctx.sender().to_id(),
            );
            assert!(
                accreditations_to_accredit.are_statements_compliant(
                    &wanted_statements,
                    current_time_ms,
                ),
                EUnauthorizedInsufficientAccreditationToAccredit,
            );
        };

        let accredited_statement = accreditation::new_accreditation(wanted_statements, ctx);

        if (self.governance.accreditations_to_attest.contains(&receiver)) {
            self
                .governance
                .accreditations_to_attest
                .get_mut(&receiver)
                .add_accreditation(accredited_statement);
        } else {
            let mut accreditations_to_attest = accreditation::new_empty_accreditations();
            accreditations_to_attest.add_accreditation(accredited_statement);
            self.governance.accreditations_to_attest.insert(receiver, accreditations_to_attest);

            // Create and transfer capability
            transfer::transfer(self.new_cap_attest(ctx), receiver.to_address());
        };
    }

    /// Revokes attestation rights from an entity
    public fun revoke_accreditation_to_attest(
        self: &mut Federation,
        cap: &AttestCap,
        entity_id: &ID,
        permission_id: &ID,
        ctx: &mut TxContext,
    ) {
        let current_time_ms = ctx.epoch_timestamp_ms();
        assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

        let remover_accreditations = self.get_accreditations_to_attest(&ctx.sender().to_id());
        let entitys_attest_permissions = self.get_accreditations_to_attest(entity_id);
        let mut accreditation_to_revoke_idx = entitys_attest_permissions.find_accredited_statement_id(
            permission_id,
        );
        assert!(accreditation_to_revoke_idx.is_some(), EAccreditationNotFound);

        // Check revocation permissions
        if (!self.is_root_authority(&ctx.sender().to_id())) {
            let accreditation_to_revoke =
                &entitys_attest_permissions.accredited_statements()[
                    accreditation_to_revoke_idx.extract(),
                ];
            let (_, statements) = (*accreditation_to_revoke.statements()).into_keys_values();
            assert!(
                remover_accreditations.are_statements_compliant(&statements, current_time_ms),
                EUnauthorizedInsufficientAccreditationToAttest,
            );
        };

        // Remove the permission
        let entitys_attest_permissions = self.governance.accreditations_to_attest.get_mut(entity_id);
        entitys_attest_permissions.remove_accredited_statement(permission_id);
    }

    /// Revokes accreditation rights from an entity
    public fun revoke_accreditation_to_accredit(
        self: &mut Federation,
        cap: &AccreditCap,
        entity_id: &ID,
        permission_id: &ID,
        ctx: &mut TxContext,
    ) {
        assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
        let remover_permissions = self.get_accreditations_to_accredit(&ctx.sender().to_id());

        let entitys_accredit_permissions = self.get_accreditations_to_accredit(entity_id);
        let mut accreditation_to_revoke_idx = entitys_accredit_permissions.find_accredited_statement_id(
            permission_id,
        );
        assert!(accreditation_to_revoke_idx.is_some(), EAccreditationNotFound);

        // Check revocation permissions
        if (!self.is_root_authority(&ctx.sender().to_id())) {
            let accreditation_to_revoke =
                &entitys_accredit_permissions.accredited_statements()[
                    accreditation_to_revoke_idx.extract(),
                ];
            let current_time_ms = ctx.epoch_timestamp_ms();
            let (_, statements) = (*accreditation_to_revoke.statements()).into_keys_values();
            assert!(
                remover_permissions.are_statements_compliant(&statements, current_time_ms),
                EUnauthorizedInsufficientAccreditationToAccredit,
            );
        };

        // Remove the permission
        let entitys_accredit_permissions = self
            .governance
            .accreditations_to_accredit
            .get_mut(entity_id);
        entitys_accredit_permissions.remove_accredited_statement(permission_id);
    }

    // ===== Validation Functions =====

    /// Validates a single statement from an attester
    public fun validate_statement(
        self: &Federation,
        attester_id: &ID,
        statement_name: StatementName,
        statement_value: StatementValue,
        ctx: &mut TxContext,
    ) {
        let current_time_ms = ctx.epoch_timestamp_ms();
        assert!(self.is_statement_in_federation(statement_name), EInvalidStatement);

        let accreditations = self.get_accreditations_to_attest(attester_id);
        assert!(
            accreditations.is_statement_allowed(&statement_name, &statement_value, current_time_ms),
            EAttesterInsufficientAccreditation,
        );
    }

    /// Validates multiple statements from an issuer
    public fun validate_statements(
        self: &Federation,
        attester_id: &ID,
        statements: VecMap<StatementName, StatementValue>,
        ctx: &mut TxContext,
    ) {
        let current_time_ms = ctx.epoch_timestamp_ms();
        let statement_names = statements.keys();

        // First check if all statements are trusted by the federation
        let mut idx = 0;
        while (idx < statement_names.length()) {
            let statement_name = statement_names[idx];
            assert!(self.is_statement_in_federation(statement_name), EInvalidStatement);
            idx = idx + 1;
        };

        // Then check if issuer has permissions for all statements
        let accreditations = self.get_accreditations_to_attest(attester_id);
        assert!(
            accreditations.are_statements_allowed(&statements, current_time_ms),
            EAttesterInsufficientAccreditation,
        );
    }

    // ===== Helper Functions =====

    /// Checks if an entity is a root authority in the federation
    fun is_root_authority(self: &Federation, id: &ID): bool {
        let mut idx = 0;
        while (idx < self.root_authorities.length()) {
            if (self.root_authorities[idx].account_id == *id) {
                return true
            };
            idx = idx + 1;
        };
        false
    }
}

// ===== Test Module =====

#[test_only]
module ith::main_tests {
    use iota::test_scenario;
    use iota::vec_set;
    use ith::main::{
        new_federation,
        RootAuthorityCap,
        Federation,
        AccreditCap,
        AttestCap,
        add_statement,
        create_accreditation_to_accredit,
        create_accreditation_to_attest,
        revoke_accreditation_to_attest,
        revoke_accreditation_to_accredit
    };
    use ith::statement_name::new_statement_name;
    use ith::statement_value::new_statement_value_number;
    use std::string::utf8;

    #[test]
    fun creating_new_federation_works() {
        let alice = @0x1;
        let mut scenario = test_scenario::begin(alice);

        // Create new federation
        new_federation(scenario.ctx());
        scenario.next_tx(alice);

        // Verify alice has RootAuthorityCap
        let cap: RootAuthorityCap = scenario.take_from_address(alice);
        let fed: Federation = scenario.take_shared();

        assert!(fed.is_accreditor(&alice.to_id()), 0);
        assert!(fed.is_attester(&alice.to_id()), 0);

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

        // Create federation
        new_federation(scenario.ctx());
        scenario.next_tx(alice);

        let mut fed: Federation = scenario.take_shared();
        let cap: RootAuthorityCap = scenario.take_from_address(alice);

        // Add new root authority
        fed.add_root_authority(&cap, bob, scenario.ctx());
        scenario.next_tx(alice);

        // Verify bob has RootAuthorityCap
        let bob_cap: RootAuthorityCap = scenario.take_from_address(bob.to_address());

        test_scenario::return_to_address(alice, cap);
        test_scenario::return_to_address(bob.to_address(), bob_cap);
        test_scenario::return_shared(fed);
        new_object.delete();
        let _ = scenario.end();
    }

    #[test]
    fun test_adding_statement() {
        let alice = @0x1;
        let mut scenario = test_scenario::begin(alice);

        // Create federation
        new_federation(scenario.ctx());
        scenario.next_tx(alice);

        let mut fed: Federation = scenario.take_shared();
        let cap: RootAuthorityCap = scenario.take_from_address(alice);

        // Add statement
        let statement_name = new_statement_name(utf8(b"statement_name"));
        let statement_value = new_statement_value_number(10);
        let mut allowed_values = vec_set::empty();
        allowed_values.insert(statement_value);

        fed.add_statement(&cap, statement_name, allowed_values, false, scenario.ctx());
        scenario.next_tx(alice);

        // Verify statement was added
        assert!(fed.is_statement_in_federation(statement_name), 0);

        test_scenario::return_to_address(alice, cap);
        test_scenario::return_shared(fed);
        let _ = scenario.end();
    }

    #[test]
    fun test_create_accreditation() {
        let alice = @0x1;
        let mut scenario = test_scenario::begin(alice);

        // Create federation
        new_federation(scenario.ctx());
        scenario.next_tx(alice);

        let mut fed: Federation = scenario.take_shared();
        let cap: RootAuthorityCap = scenario.take_from_address(alice);
        let accredit_cap: AccreditCap = scenario.take_from_address(alice);

        let new_id = scenario.new_object();
        let bob = new_id.uid_to_inner();

        // Issue accreditation permission
        let statements = vector::empty();
        fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, scenario.ctx());
        scenario.next_tx(alice);

        // Verify permission was issued
        assert!(fed.is_accreditor(&bob), 0);

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

        // Create federation
        new_federation(scenario.ctx());
        scenario.next_tx(alice);

        let mut fed: Federation = scenario.take_shared();
        let cap: RootAuthorityCap = scenario.take_from_address(alice);
        let attest_cap: AttestCap = scenario.take_from_address(alice);

        let new_id = scenario.new_object();
        let bob = new_id.uid_to_inner();

        // Issue attestation permission
        let statements = vector::empty();
        fed.create_accreditation_to_attest(&attest_cap, bob, statements, scenario.ctx());
        scenario.next_tx(alice);

        // Verify permission was issued
        assert!(fed.is_attester(&bob), 0);

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

        // Create federation
        new_federation(scenario.ctx());
        scenario.next_tx(alice);

        let mut fed: Federation = scenario.take_shared();
        let cap: RootAuthorityCap = scenario.take_from_address(alice);
        let accredit_cap: AccreditCap = scenario.take_from_address(alice);
        let attest_cap: AttestCap = scenario.take_from_address(alice);

        let new_id = scenario.new_object();
        let bob = new_id.uid_to_inner();

        // Issue permissions
        let statements = vector::empty();
        fed.create_accreditation_to_accredit(&accredit_cap, bob, statements, scenario.ctx());
        scenario.next_tx(alice);

        fed.create_accreditation_to_attest(&attest_cap, bob, statements, scenario.ctx());
        scenario.next_tx(alice);

        // Revoke attestation permission
        let permission_id = fed
            .get_accreditations_to_attest(&bob)
            .accredited_statements()[0]
            .id()
            .uid_to_inner();
        fed.revoke_accreditation_to_attest(&attest_cap, &bob, &permission_id, scenario.ctx());
        scenario.next_tx(alice);

        // Revoke accreditation permission
        let permission_id = fed
            .get_accreditations_to_accredit(&bob)
            .accredited_statements()[0]
            .id()
            .uid_to_inner();
        fed.revoke_accreditation_to_accredit(&accredit_cap, &bob, &permission_id, scenario.ctx());
        scenario.next_tx(alice);

        // TODO: Fix - entitys should not have permissions after revocation
        assert!(fed.is_attester(&bob), 0);
        assert!(fed.is_accreditor(&bob), 0);

        test_scenario::return_to_address(alice, cap);
        test_scenario::return_to_address(alice, accredit_cap);
        test_scenario::return_to_address(alice, attest_cap);
        test_scenario::return_shared(fed);
        new_id.delete();
        let _ = scenario.end();
    }
}
