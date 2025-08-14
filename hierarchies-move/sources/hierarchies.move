/// This module implements a hierarchical trust system where entities can delegate
/// trust and attestation rights to other entities within a federation.
module hierarchies::main;

use hierarchies::{
    accreditation::{Self, Accreditations},
    statement::{Self, Statements, Statement},
    statement_name::StatementName,
    statement_value::StatementValue
};
use iota::{clock::Clock, event, vec_map::{Self, VecMap}, vec_set::VecSet};

// ===== Errors =====
/// Error when operation is performed with wrong federation
const EUnauthorizedWrongFederation: u64 = 1;
/// Error when entity lacks sufficient accreditation permissions
const EUnauthorizedInsufficientAccreditationToAccredit: u64 = 2;
/// Error when Value Condition for Statement is invalid (e.g., allow_any=true with specific values)
const EInvalidStatementValueCondition: u64 = 3;
/// Error when trying to access non-existent accreditation
const EAccreditationNotFound: u64 = 4;
/// Error when timestamp is in the past
const ETimestampMustBeInTheFuture: u64 = 5;
/// Error when trying to create accreditation for statement not in federation
const EStatementNotInFederation: u64 = 6;
/// Error when trying to revoke non-existent root authority
const ERootAuthorityNotFound: u64 = 7;
/// Error when trying to revoke the last root authority
const ECannotRevokeLastRootAuthority: u64 = 8;
/// Error when using a revoked root authority capability
const ERevokedRootAuthority: u64 = 9;
/// Empty allowed values list without allow_any flag
const EEmptyAllowedValuesWithoutAllowAny: u64 = 10;
/// Error when trying to add an already existing root authority
const EAlreadyRootAuthority: u64 = 11;
/// Error when trying to reinstate a root authority that is not revoked
const ENotRevokedRootAuthority: u64 = 12;

// ===== Constants =====
const TIME_BUFFER_MS: u64 = 5000;

// ===== Core Data Structures =====

/// The main federation object representing a hierarchy of trust.
/// This is a shared object that maintains the trust structure and governance.
public struct Federation has key, store {
    id: UID,
    governance: Governance,
    root_authorities: vector<RootAuthority>,
    revoked_root_authorities: vector<ID>,
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
    account_id: ID,
}

/// Capability for accreditation operations
public struct AccreditCap has key {
    id: UID,
    federation_id: ID,
}

// ===== Event Structures =====

/// Event emitted when a new federation is created
public struct FederationCreatedEvent has copy, drop {
    federation_address: address,
}

/// Event emitted when a statement is added to the federation
public struct StatementAddedEvent has copy, drop {
    federation_address: address,
    statement_name: StatementName,
    allow_any: bool,
}

/// Event emitted when a statement is revoked
public struct StatementRevokedEvent has copy, drop {
    federation_address: address,
    statement_name: StatementName,
    valid_to_ms: u64,
}

/// Event emitted when a root authority is added
public struct RootAuthorityAddedEvent has copy, drop {
    federation_address: address,
    account_id: ID,
}

/// Event emitted when a root authority is revoked
public struct RootAuthorityRevokedEvent has copy, drop {
    federation_address: address,
    account_id: ID,
}

/// Event emitted when a root authority is reinstated
public struct RootAuthorityReinstatedEvent has copy, drop {
    federation_address: address,
    account_id: ID,
    reinstated_by: ID,
}

/// Event emitted when accreditation to accredit is created
public struct AccreditationToAccreditCreatedEvent has copy, drop {
    federation_address: address,
    receiver: ID,
    accreditor: ID,
}

/// Event emitted when accreditation to attest is created
public struct AccreditationToAttestCreatedEvent has copy, drop {
    federation_address: address,
    receiver: ID,
    accreditor: ID,
}

/// Event emitted when accreditation to attest is revoked
public struct AccreditationToAttestRevokedEvent has copy, drop {
    federation_address: address,
    entity_id: ID,
    permission_id: ID,
    revoker: ID,
}

/// Event emitted when accreditation to accredit is revoked
public struct AccreditationToAccreditRevokedEvent has copy, drop {
    federation_address: address,
    entity_id: ID,
    permission_id: ID,
    revoker: ID,
}

// ===== Constructor Functions =====

/// Creates a new federation with the sender as the first root authority.
/// The creator of the Federation becomes the root authority of the Federation.
public fun new_federation(ctx: &mut TxContext) {
    let federation_id = object::new(ctx);
    let mut federation = Federation {
        id: federation_id,
        root_authorities: vector::empty(),
        revoked_root_authorities: vector::empty(),
        governance: Governance {
            id: object::new(ctx),
            statements: statement::new_statements(),
            accreditations_to_accredit: vec_map::empty(),
            accreditations_to_attest: vec_map::empty(),
        },
    };

    // Create root authority and capabilities
    let root_auth_cap = new_root_authority_cap(&federation, ctx.sender().to_id(), ctx);
    let root_authority = new_root_authority(ctx.sender().to_id(), ctx);
    vector::push_back(&mut federation.root_authorities, root_authority);

    // Grant permissions to the creator
    let permission = accreditation::new_empty_accreditations();
    federation.governance.accreditations_to_accredit.insert(ctx.sender().to_id(), permission);

    let permission = accreditation::new_empty_accreditations();
    federation.governance.accreditations_to_attest.insert(ctx.sender().to_id(), permission);

    // Create and transfer capabilities
    let accredit_cap = new_cap_accredit(&federation, ctx);

    // Emit federation created event
    event::emit(FederationCreatedEvent {
        federation_address: object::uid_to_address(&federation.id),
    });

    // Transfer capabilities to creator
    transfer::transfer(root_auth_cap, ctx.sender());
    transfer::transfer(accredit_cap, ctx.sender());

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
fun new_root_authority_cap(
    self: &Federation,
    account_id: ID,
    ctx: &mut TxContext,
): RootAuthorityCap {
    RootAuthorityCap {
        id: object::new(ctx),
        federation_id: self.federation_id(),
        account_id,
    }
}

/// Creates a new accreditation capability
fun new_cap_accredit(self: &Federation, ctx: &mut TxContext): AccreditCap {
    AccreditCap {
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
    _: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    assert!(!self.is_revoked_root_authority(&cap.account_id), ERevokedRootAuthority);
    assert!(!(allow_any && allowed_values.keys().length() > 0), EInvalidStatementValueCondition);
    assert!(allow_any || allowed_values.keys().length() > 0, EEmptyAllowedValuesWithoutAllowAny);

    let statement = statement::new_statement(
        statement_name,
        allowed_values,
        allow_any,
        option::none(),
    );

    self.governance.statements.add_statement(statement);

    // Emit statement added event
    event::emit(StatementAddedEvent {
        federation_address: self.federation_id().to_address(),
        statement_name,
        allow_any,
    });
}

/// Revokes a statement by setting its validity period
public fun revoke_statement(
    federation: &mut Federation,
    cap: &RootAuthorityCap,
    statement_name: StatementName,
    clock: &Clock,
    _: &mut TxContext,
) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);
    assert!(!federation.is_revoked_root_authority(&cap.account_id), ERevokedRootAuthority);
    let statement = federation.governance.statements.data_mut().get_mut(&statement_name);
    statement.revoke(clock.timestamp_ms());

    // Emit statement revoked event
    event::emit(StatementRevokedEvent {
        federation_address: federation.federation_id().to_address(),
        statement_name,
        valid_to_ms: clock.timestamp_ms(),
    });
}

/// Revokes a statement by setting its validity period
public fun revoke_statement_at(
    federation: &mut Federation,
    cap: &RootAuthorityCap,
    statement_name: StatementName,
    valid_to_ms: u64,
    clock: &Clock,
    _: &mut TxContext,
) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);
    assert!(!federation.is_revoked_root_authority(&cap.account_id), ERevokedRootAuthority);
    assert!(valid_to_ms > clock.timestamp_ms() + TIME_BUFFER_MS, ETimestampMustBeInTheFuture);
    let statement = federation.governance.statements.data_mut().get_mut(&statement_name);
    statement.revoke(valid_to_ms);

    // Emit statement revoked event
    event::emit(StatementRevokedEvent {
        federation_address: federation.federation_id().to_address(),
        statement_name,
        valid_to_ms,
    });
}

/// Adds a new root authority to the federation.
/// Only existing root authorities can perform this operation.
public fun add_root_authority(
    self: &mut Federation,
    cap: &RootAuthorityCap,
    account_id: ID,
    ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    assert!(!self.is_root_authority(&account_id), EAlreadyRootAuthority);
    assert!(!self.is_revoked_root_authority(&cap.account_id), ERevokedRootAuthority);

    let root_authority = new_root_authority(account_id, ctx);
    vector::push_back(&mut self.root_authorities, root_authority);

    let cap = new_root_authority_cap(self, account_id, ctx);
    transfer::transfer(cap, account_id.to_address());

    event::emit(RootAuthorityAddedEvent {
        federation_address: self.federation_id().to_address(),
        account_id,
    });
}

/// Revokes a root authority from the federation.
/// Only root authorities can perform this operation.
/// Cannot revoke the last root authority to prevent lockout.
public fun revoke_root_authority(
    self: &mut Federation,
    cap: &RootAuthorityCap,
    account_id: ID,
    _: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    assert!(self.is_root_authority(&account_id), ERootAuthorityNotFound);

    assert!(self.root_authorities.length() > 1, ECannotRevokeLastRootAuthority);

    // Find and revoke the root authority
    let mut idx = 0;
    let mut found = false;
    while (idx < self.root_authorities.length()) {
        if (self.root_authorities[idx].account_id == account_id) {
            let RootAuthority { id, account_id: removed_id } = vector::remove(
                &mut self.root_authorities,
                idx,
            );
            object::delete(id);

            // Add to revocation list
            vector::push_back(&mut self.revoked_root_authorities, removed_id);

            found = true;
            break
        };
        idx = idx + 1;
    };

    assert!(found, ERootAuthorityNotFound);

    event::emit(RootAuthorityRevokedEvent {
        federation_address: self.federation_id().to_address(),
        account_id,
    });
}

/// Reinstates a previously revoked root authority to the federation.
/// Only existing root authorities can perform this operation.
/// The account must be in the revoked list to be reinstated.
public fun reinstate_root_authority(
    self: &mut Federation,
    cap: &RootAuthorityCap,
    account_id: ID,
    ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    assert!(!self.is_root_authority(&account_id), EAlreadyRootAuthority);

    let mut idx = 0;
    let mut found = false;
    while (idx < self.revoked_root_authorities.length()) {
        if (self.revoked_root_authorities[idx] == account_id) {
            vector::remove(&mut self.revoked_root_authorities, idx);
            found = true;
            break
        };
        idx = idx + 1;
    };

    assert!(found, ENotRevokedRootAuthority);

    let root_authority = new_root_authority(account_id, ctx);
    vector::push_back(&mut self.root_authorities, root_authority);

    event::emit(RootAuthorityReinstatedEvent {
        federation_address: self.federation_id().to_address(),
        account_id,
        reinstated_by: ctx.sender().to_id(),
    });
}

/// Grants accreditation rights to another entity.
/// Allows the receiver to delegate accreditation permissions to others.
public fun create_accreditation_to_accredit(
    self: &mut Federation,
    cap: &AccreditCap,
    receiver: ID,
    want_statements: vector<Statement>,
    clock: &Clock,
    ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    let current_time_ms = clock.timestamp_ms();

    // Validate that all statement names exist in federation
    let mut idx = 0;
    while (idx < want_statements.length()) {
        let statement = &want_statements[idx];
        assert!(
            self.is_statement_in_federation(*statement.statement_name()),
            EStatementNotInFederation,
        );
        idx = idx + 1;
    };

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
    };
    event::emit(AccreditationToAccreditCreatedEvent {
        federation_address: self.federation_id().to_address(),
        receiver,
        accreditor: ctx.sender().to_id(),
    });
}

/// Grants attestation rights to another entity.
/// Allows the receiver to create trusted attestations.
public fun create_accreditation_to_attest(
    self: &mut Federation,
    cap: &AccreditCap,
    receiver: ID,
    wanted_statements: vector<Statement>,
    clock: &Clock,
    ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    let current_time_ms = clock.timestamp_ms();

    // Validate that all statement names exist in federation
    let mut idx = 0;
    while (idx < wanted_statements.length()) {
        let statement = &wanted_statements[idx];
        assert!(
            self.is_statement_in_federation(*statement.statement_name()),
            EStatementNotInFederation,
        );
        idx = idx + 1;
    };

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
    };

    event::emit(AccreditationToAttestCreatedEvent {
        federation_address: self.federation_id().to_address(),
        receiver,
        accreditor: ctx.sender().to_id(),
    });
}

/// Revokes attestation rights from an entity
public fun revoke_accreditation_to_attest(
    self: &mut Federation,
    cap: &AccreditCap,
    entity_id: &ID,
    permission_id: &ID,
    clock: &Clock,
    ctx: &mut TxContext,
) {
    let current_time_ms = clock.timestamp_ms();
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let remover_accreditations = self.get_accreditations_to_accredit(&ctx.sender().to_id());
    let entities_attest_permissions = self.get_accreditations_to_attest(entity_id);
    let mut accreditation_to_revoke_idx = entities_attest_permissions.find_accredited_statement_id(
        permission_id,
    );
    assert!(accreditation_to_revoke_idx.is_some(), EAccreditationNotFound);

    // Check revocation permissions
    if (!self.is_root_authority(&ctx.sender().to_id())) {
        let accreditation_to_revoke =
            &entities_attest_permissions.accredited_statements()[
                accreditation_to_revoke_idx.extract(),
            ];
        let (_, statements) = (*accreditation_to_revoke.statements()).into_keys_values();
        assert!(
            remover_accreditations.are_statements_compliant(&statements, current_time_ms),
            EUnauthorizedInsufficientAccreditationToAccredit,
        );
    };

    let entities_attest_permissions = self.governance.accreditations_to_attest.get_mut(entity_id);
    entities_attest_permissions.remove_accredited_statement(permission_id);

    event::emit(AccreditationToAttestRevokedEvent {
        federation_address: self.federation_id().to_address(),
        entity_id: *entity_id,
        permission_id: *permission_id,
        revoker: ctx.sender().to_id(),
    });
}

/// Revokes accreditation rights from an entity
public fun revoke_accreditation_to_accredit(
    self: &mut Federation,
    cap: &AccreditCap,
    entity_id: &ID,
    permission_id: &ID,
    clock: &Clock,
    ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    let remover_permissions = self.get_accreditations_to_accredit(&ctx.sender().to_id());

    let entities_accredit_permissions = self.get_accreditations_to_accredit(entity_id);
    let mut accreditation_to_revoke_idx = entities_accredit_permissions.find_accredited_statement_id(
        permission_id,
    );
    assert!(accreditation_to_revoke_idx.is_some(), EAccreditationNotFound);

    // Check revocation permissions
    if (!self.is_root_authority(&ctx.sender().to_id())) {
        let accreditation_to_revoke =
            &entities_accredit_permissions.accredited_statements()[
                accreditation_to_revoke_idx.extract(),
            ];
        let current_time_ms = clock.timestamp_ms();
        let (_, statements) = (*accreditation_to_revoke.statements()).into_keys_values();
        assert!(
            remover_permissions.are_statements_compliant(&statements, current_time_ms),
            EUnauthorizedInsufficientAccreditationToAccredit,
        );
    };

    let entities_accredit_permissions = self
        .governance
        .accreditations_to_accredit
        .get_mut(entity_id);
    entities_accredit_permissions.remove_accredited_statement(permission_id);

    event::emit(AccreditationToAccreditRevokedEvent {
        federation_address: self.federation_id().to_address(),
        entity_id: *entity_id,
        permission_id: *permission_id,
        revoker: ctx.sender().to_id(),
    });
}

// ===== Validation Functions =====

/// Validates a single statement from an attester
/// Returns true if validation passes, false otherwise
public fun validate_statement(
    self: &Federation,
    attester_id: &ID,
    statement_name: StatementName,
    statement_value: StatementValue,
    clock: &Clock,
): bool {
    let current_time_ms = clock.timestamp_ms();

    // Check if statement is trusted by the federation
    if (!self.is_statement_in_federation(statement_name)) {
        return false
    };

    // Check if attester has permissions for the statement
    let accreditations = self.get_accreditations_to_attest(attester_id);
    if (!accreditations.is_statement_allowed(&statement_name, &statement_value, current_time_ms)) {
        return false
    };

    true
}

/// Validates multiple statements from an issuer
/// Returns true if all validations pass, false otherwise
public fun validate_statements(
    self: &Federation,
    attester_id: &ID,
    statements: VecMap<StatementName, StatementValue>,
    clock: &Clock,
): bool {
    let current_time_ms = clock.timestamp_ms();
    let statement_names = statements.keys();

    // First check if all statements are trusted by the federation
    let mut idx = 0;
    while (idx < statement_names.length()) {
        let statement_name = statement_names[idx];
        if (!self.is_statement_in_federation(statement_name)) {
            return false
        };
        idx = idx + 1;
    };

    // Then check if issuer has permissions for all statements
    let accreditations = self.get_accreditations_to_attest(attester_id);
    if (!accreditations.are_statements_allowed(&statements, current_time_ms)) {
        return false
    };

    true
}

/// Checks if an entity is a root authority in the federation
public fun is_root_authority(self: &Federation, id: &ID): bool {
    let mut idx = 0;
    if (self.is_revoked_root_authority(id)) {
        return false
    };

    while (idx < self.root_authorities.length()) {
        if (self.root_authorities[idx].account_id == *id) {
            return true
        };
        idx = idx + 1;
    };
    false
}

/// Checks if an entity is a revoked root authority
fun is_revoked_root_authority(self: &Federation, id: &ID): bool {
    vector::contains(&self.revoked_root_authorities, id)
}

// ===== Test Functions =====
#[test_only]
public(package) fun transfer_root_authority_cap(
    self: &Federation,
    cap: RootAuthorityCap,
    account_id: ID,
    _: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    assert!(!self.is_revoked_root_authority(&cap.account_id), ERevokedRootAuthority);

    transfer::transfer(cap, account_id.to_address());
}
