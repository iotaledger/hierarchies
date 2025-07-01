// HTF Notary module
module ith::main;

use iota::{clock::Clock, event, vec_map::{Self, VecMap}, vec_set::VecSet};
use ith::{
    accreditation::{Self, Accreditations},
    statement::{Self, Statements, Statement},
    statement_name::StatementName,
    statement_value::StatementValue
};

const EUnauthorizedWrongFederation: u64 = 1;
const EUnauthorizedInsufficientAccreditationToAccredit: u64 = 2;
const EUnauthorizedInsufficientAccreditationToAccreditToAttest: u64 = 3;
const EInvalidStatement: u64 = 4;
const EInvalidEntityInsufficientAccreditation: u64 = 5;
const EInvalidStatementCondition: u64 = 6;
const EAccreditationNotFound: u64 = 7;

public struct Event<D> has copy, drop {
    data: D,
}

public struct FederationCreatedEvent has copy, drop {
    federation_address: address,
}

public struct RootAuthorityCap has key { id: UID, federation_id: ID }
public struct AttestCap has key { id: UID, federation_id: ID }
public struct AccreditCap has key { id: UID, federation_id: ID }

// Federation is the hierarchy of trust in the system. It's a public, shared object
public struct Federation has key, store {
    id: UID,
    governance: Governance,
    root_authorities: vector<RootAuthority>,
}

// Root authority has the highest trust in the system, it can delegate trust
// to other entities and itself
public struct RootAuthority has key, store {
    id: UID,
    account_id: ID,
}

// Governance is the object that contains the Statements and tracks the
// abilities of entities to attest and accredit
public struct Governance has key, store {
    id: UID,
    /// Statements that are trusted by the given Federation
    statements: Statements,
    /// Accreditation rights to delegate accreditation.
    /// These accreditations allow a user to grant other users the ability to further delegate accreditation permissions.
    accreditations_to_accredit: VecMap<ID, Accreditations>,
    /// Accreditation rights for attestation.
    /// These accreditations empower a user to create Attestations.
    accreditations_to_attest: VecMap<ID, Accreditations>,
}

/// Checks if the given ID is a root authority of the Federation.
fun is_root_authority(self: &Federation, id: &ID): bool {
    let mut idx = 0;
    while (idx < self.root_authorities.length()) {
        if (self.root_authorities[idx].account_id == id) {
            return true
        };
        idx = idx + 1;
    };
    false
}

/// Creates a new Federation object and returns it.
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

    event::emit(Event {
        data: FederationCreatedEvent {
            federation_address: federation.federation_id().to_address(),
        },
    });

    transfer::transfer(root_auth_cap, ctx.sender());
    transfer::transfer(accredit_cap, ctx.sender());
    transfer::transfer(attest_cap, ctx.sender());

    transfer::share_object(federation)
}

/// Adds a new Statement to the Federation along with the allowed values and constraints.
public fun add_statement(
    self: &mut Federation,
    cap: &RootAuthorityCap,
    statement_name: StatementName,
    allowed_values: VecSet<StatementValue>,
    allow_any: bool,
    _ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    assert!(!(allow_any && allowed_values.keys().length() > 0), EInvalidStatementCondition);

    let statement = statement::new_statement(
        statement_name,
        allowed_values,
        allow_any,
        option::none(),
    );

    self.governance.statements.add_statement(statement);
}

// Revokes Statement by setting the validity to a specific time
public fun revoke_statement(
    federation: &mut Federation,
    cap: &RootAuthorityCap,
    statement_name: StatementName,
    valid_to_ms: u64,
) {
    assert!(cap.federation_id == federation.federation_id(), EUnauthorizedWrongFederation);
    let property_statement = federation.governance.statements.data_mut().get_mut(&statement_name);
    property_statement.revoke(valid_to_ms);
}

/// Validates if the given entity can attest the given Statement.
public fun validate_statement(
    self: &Federation,
    attester_id: &ID,
    statement_name: StatementName,
    statement_value: StatementValue,
    clock: &Clock,
) {
    let current_time_ms = clock.timestamp_ms();
    assert!(self.is_statement_in_federation(statement_name), EInvalidStatement);

    let accreditations = self.get_accreditations_to_attest(attester_id);
    assert!(
        accreditations.is_statement_allowed(&statement_name, &statement_value, current_time_ms),
        EInvalidEntityInsufficientAccreditation,
    );
}

/// Validates if the given entity can attest the given Statements.
public fun validate_statements(
    self: &Federation,
    entity_id: &ID,
    statements: VecMap<StatementName, StatementValue>,
    clock: &Clock,
) {
    let current_time_ms = clock.timestamp_ms();
    let statement_names = statements.keys();

    let mut idx = 0;
    while (idx < statement_names.length()) {
        let statement_name = statement_names[idx];
        assert!(self.is_statement_in_federation(statement_name), EInvalidStatement);
        idx = idx + 1;
    };
    // then check if names and values are permitted for given entity
    let accreditations = self.get_accreditations_to_attest(entity_id);
    assert!(
        accreditations.are_statements_allowed(&statements, current_time_ms),
        EInvalidEntityInsufficientAccreditation,
    );
}

/// Adds a new root authority to the Federation.
public fun add_root_authority(
    self: &mut Federation,
    cap: &RootAuthorityCap,
    account_id: ID,
    ctx: &mut TxContext,
) {
    if (cap.federation_id != self.federation_id()) {
        abort EUnauthorizedWrongFederation
    };
    let root_authority = Self::new_root_authority(account_id, ctx);
    vector::push_back(&mut self.root_authorities, root_authority);

    let cap = Self::new_root_authority_cap(self, ctx);
    transfer::transfer(cap, account_id.to_address());
}

/// Accredit to attest is a a process when trusted entity can make make another entity the Attester. Attester can make statements that are trusted by the federation.
public fun accredit_to_attest(
    self: &mut Federation,
    cap: &AttestCap,
    receiver: ID,
    wanted_statements: vector<Statement>,
    ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);

    let current_time_ms = ctx.epoch_timestamp_ms();

    // Check the permissions only if the sender is not a root authority
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

        // also create a capability
        transfer::transfer(self.new_cap_attest(ctx), receiver.to_address());
    };
}

/// Revokes the accreditation to attest for an entity.
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

    let users_attest_permissions = self.get_accreditations_to_attest(entity_id);
    let mut accreditation_to_revoke_idx = users_attest_permissions.find_accredited_statement_id(
        permission_id,
    );
    assert!(accreditation_to_revoke_idx.is_some(), EAccreditationNotFound);

    // Make sure that the sender has the right to revoke the permission
    if (!self.is_root_authority(&ctx.sender().to_id())) {
        let accreditation_to_revoke =
            &users_attest_permissions.accredited_statements()[
                accreditation_to_revoke_idx.extract(),
            ];
        let (_, statements) = (*accreditation_to_revoke.statements()).into_keys_values();
        assert!(
            remover_accreditations.are_statements_compliant(&statements, current_time_ms),
            EUnauthorizedInsufficientAccreditationToAccreditToAttest,
        );
    };

    // Remove the permission
    let users_attest_permissions = self.governance.accreditations_to_attest.get_mut(entity_id);
    users_attest_permissions.remove_accredited_statement(permission_id);
}

// Accredit is a process of transferring trust to another entity. It allows the entity to accredit other entities.
public fun accredit(
    self: &mut Federation,
    cap: &AccreditCap,
    receiver: ID,
    want_statements: vector<Statement>,
    ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    let current_time_ms = ctx.epoch_timestamp_ms();

    // Check the permissions only if the sender is not a root authority
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

        // also create a capability
        transfer::transfer(self.new_cap_accredit(ctx), receiver.to_address());
    }
}

/// Revokes the accreditation to accredit for an entity.
public fun revoke_accreditation_to_accredit(
    self: &mut Federation,
    cap: &AccreditCap,
    entity_id: &ID,
    permission_id: &ID,
    ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    let remover_permissions = self.get_accreditations_to_accredit(&ctx.sender().to_id());

    let users_accredit_permissions = self.get_accreditations_to_accredit(entity_id);
    let mut accreditation_to_revoke_idx = users_accredit_permissions.find_accredited_statement_id(
        permission_id,
    );
    assert!(accreditation_to_revoke_idx.is_some(), EAccreditationNotFound);

    // Make sure that the sender has the right to revoke the permission
    if (!self.is_root_authority(&ctx.sender().to_id())) {
        let accreditation_to_revoke =
            &users_accredit_permissions.accredited_statements()[
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
    let users_accredit_permissions = self.governance.accreditations_to_accredit.get_mut(entity_id);
    users_accredit_permissions.remove_accredited_statement(permission_id);
}

/// Returns the names of the statements that can be attested by the Federation.
public fun get_statements(self: &Federation): vector<StatementName> {
    self.governance.statements.data().keys()
}

/// Returns the names of the statements that can be attested by the Federation.
public fun is_statement_in_federation(self: &Federation, statement_name: StatementName): bool {
    self.governance.statements.data().contains(&statement_name)
}

/// Returns the accreditations to attest for the given entity.
public fun get_accreditations_to_attest(self: &Federation, entity_id: &ID): &Accreditations {
    self.governance.accreditations_to_attest.get(entity_id)
}

/// Checks if the given entity is an attester in the Federation.
public fun is_attester(self: &Federation, entity_id: &ID): bool {
    self.governance.accreditations_to_attest.contains(entity_id)
}

/// Returns the accreditations to accredit for the given entity.
public fun get_accreditations_to_accredit(self: &Federation, entity_id: &ID): &Accreditations {
    self.governance.accreditations_to_accredit.get(entity_id)
}

/// Checks if the given entity is an accreditor in the Federation.
public fun is_accreditor(self: &Federation, entity_id: &ID): bool {
    self.governance.accreditations_to_accredit.contains(entity_id)
}

/// Creates a new RootAuthority object with the given account ID.
public fun new_root_authority(account_id: ID, ctx: &mut TxContext): RootAuthority {
    RootAuthority {
        id: object::new(ctx),
        account_id: account_id,
    }
}

fun new_cap_attest(self: &Federation, ctx: &mut TxContext): AttestCap {
    AttestCap {
        id: object::new(ctx),
        federation_id: self.federation_id(),
    }
}

fun new_cap_accredit(self: &Federation, ctx: &mut TxContext): AccreditCap {
    AccreditCap {
        id: object::new(ctx),
        federation_id: self.federation_id(),
    }
}

fun new_root_authority_cap(self: &Federation, ctx: &mut TxContext): RootAuthorityCap {
    RootAuthorityCap {
        id: object::new(ctx),
        federation_id: self.federation_id(),
    }
}

fun federation_id(self: &Federation): ID {
    self.id.to_inner()
}

// TODO should be removed
// This has left due to of compatibility with the rust abstraction
public fun remove_statement(
    self: &mut Federation,
    cap: &RootAuthorityCap,
    statement_name: StatementName,
    _ctx: &mut TxContext,
) {
    assert!(cap.federation_id == self.federation_id(), EUnauthorizedWrongFederation);
    self.governance.statements.data_mut().remove(&statement_name);
}
