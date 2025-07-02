// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use async_trait::async_trait;
use iota_interaction::move_types::language_storage::StructTag;
use iota_interaction::rpc_types::{IotaObjectDataFilter, IotaObjectDataOptions, IotaObjectResponseQuery};
use iota_interaction::types::base_types::{IotaAddress, ObjectID, ObjectRef, SequenceNumber};
use iota_interaction::types::object::Owner;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::{CallArg, ObjectArg, ProgrammableTransaction};
use iota_interaction::{ident_str, IotaClientTrait, MoveType, OptionalSync};
use iota_sdk::types::transaction::Command;
use product_common::core_client::CoreClientReadOnly;

use crate::core::types::statements::name::StatementName;
use crate::core::types::statements::value::StatementValue;
use crate::core::types::statements::{new_property_statement, Statement};
use crate::core::types::Capability;
use crate::core::types::{
    new_statement, new_statement_name, new_statement_value_number, new_statement_value_string, Capability, Statement,
    StatementName, StatementValue,
};
use crate::error::Error;
use crate::utils::{self};

const MAIN_ITH_MODULE: &str = move_names::MODULE_MAIN;
pub mod move_names {
    pub const PACKAGE_NAME: &str = "ith";
    pub const MODULE_MAIN: &str = "main";
    pub const MODULE_STATEMENT: &str = "statement";
    pub const MODULE_VALUE: &str = "statement_value";
    pub const MODULE_NAME: &str = "statement_name";
    pub const MODULE_CONDITION: &str = "statement_condition";
    pub const MODULE_UTILS: &str = "utils";
}

/// Internal implementation of ITH operations.
///
/// This struct provides low-level operations for interacting with the ITH (IOTA Trust Hierarchy) module.
/// It handles capability management, federation references, and transaction building.
#[derive(Debug, Clone)]
pub(crate) struct ITHImpl;

impl ITHImpl {
    /// Retrieves a capability object for the specified sender.
    ///
    /// Capabilities grant permissions within the ITH system:
    /// - `RootAuthority`: Full administrative access
    /// - `Accredit`: Permission to delegate accreditation rights
    /// - `Attest`: Permission to create attestations
    pub(crate) async fn get_cap<C>(client: &C, cap_type: Capability, sender: IotaAddress) -> Result<ObjectRef, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let cap_tag = StructTag::from_str(&format!("{}::{MAIN_ITH_MODULE}::{cap_type}", client.package_id()))
            .map_err(|e| Error::InvalidArgument(format!("invalid cap tag: {e}")))?;

        let filter = IotaObjectResponseQuery::new_with_filter(IotaObjectDataFilter::StructType(cap_tag));

        let mut cursor = None;
        loop {
            let mut page = client
                .client_adapter()
                .read_api()
                .get_owned_objects(sender, Some(filter.clone()), cursor, None)
                .await
                .map_err(|e| Error::InvalidArgument(format!("failed to get owned objects: {e}")))?;

            let cap = std::mem::take(&mut page.data)
                .into_iter()
                .find_map(|res| res.data.map(|obj| obj.object_ref()));

            cursor = page.next_cursor;
            if let Some(cap) = cap {
                return Ok(cap);
            }
            if !page.has_next_page {
                break;
            }
        }

        Err(Error::InvalidArgument("cap not found".to_string()))
    }

    /// Creates a shared object reference for a federation.
    ///
    /// Federations are shared objects in the ITH system, requiring proper
    /// reference handling for transaction building.
    async fn get_fed_ref<C>(client: &C, federation_id: ObjectID) -> Result<ObjectArg, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let fed_ref = ObjectArg::SharedObject {
            id: federation_id,
            initial_shared_version: ITHImpl::initial_shared_version(client, &federation_id).await?,
            mutable: true,
        };

        Ok(fed_ref)
    }

    /// Retrieves the initial shared version of a shared object.
    ///
    /// Required for properly referencing shared objects in IOTA transactions.
    /// Returns an error if the object is not shared.
    pub(crate) async fn initial_shared_version<C>(client: &C, object_id: &ObjectID) -> Result<SequenceNumber, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let owner = client
            .client_adapter()
            .read_api()
            .get_object_with_options(*object_id, IotaObjectDataOptions::default().with_owner())
            .await
            .map_err(|e| Error::InvalidArgument(format!("failed to get object with options: {e}")))?
            .owner()
            .ok_or_else(|| Error::InvalidArgument("missing owner information".to_string()))?;

        match owner {
            Owner::Shared { initial_shared_version } => Ok(initial_shared_version),
            _ => Err(Error::InvalidArgument(format!(
                "object {object_id} is not a shared object"
            ))),
        }
    }
}

/// High-level ITH operations trait.
///
/// Provides methods for managing federations, statements, and accreditations.
/// Each method returns a `ProgrammableTransaction` that can be executed on-chain.
#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
pub(crate) trait ITHOperations {
    /// Creates a new federation with the caller as the initial root authority.
    ///
    /// The federation is a shared object that manages trust hierarchies.
    /// The creator receives `RootAuthorityCap`, `AccreditCap`, and `AttestCap`.
    fn new_federation(package_id: ObjectID) -> Result<ProgrammableTransaction, Error> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        ptb.move_call(
            package_id,
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("new_federation").into(),
            vec![],
            vec![],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Adds a new statement type to the federation.
    ///
    /// Statements define the types of claims that can be attested within the federation.
    /// Requires `RootAuthorityCap`. Either specify allowed values or set `allow_any` to true.
    async fn add_statement<C>(
        federation_id: ObjectID,
        statement_name: StatementName,
        allowed_values: HashSet<StatementValue>,
        allow_any: bool,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::RootAuthority, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let allow_any = ptb.pure(allow_any)?;

        let statement_names = new_statement_name(statement_name, &mut ptb, client.package_id())?;

        let value_tag = StatementValue::move_type(client.package_id());

        let mut values_of_property = vec![];
        for property_value in allowed_values {
            let value = property_value.to_ptb(&mut ptb, client.package_id())?;
        for statement_value in allowed_values {
            let value = match statement_value {
                StatementValue::Text(text) => new_statement_value_string(text, &mut ptb, client.package_id())?,
                StatementValue::Number(number) => new_statement_value_number(number, &mut ptb, client.package_id())?,
            };

            values_of_property.push(value);
        }

        let tpv_vec_set =
            utils::create_vec_set_from_move_values(values_of_property, value_tag, &mut ptb, client.package_id());

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("add_statement").into(),
            vec![],
            vec![fed_ref, cap, statement_names, tpv_vec_set, allow_any],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    async fn accredit_to_attest<C>(
        federation_id: ObjectID,
        receiver: ObjectID,
        wanted_statements: Vec<Statement>,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Attest, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let receiver = ptb.pure(receiver)?;

        let statements = new_property_statement(client.package_id(), &mut ptb, wanted_statements)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!("accredit_to_attest").into(),
            vec![],
            vec![fed_ref, cap, receiver, statements],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    async fn accredit<C>(
        federation_id: ObjectID,
        receiver: ObjectID,
        wanted_statements: Vec<Statement>,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Accredit, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let receiver = ptb.pure(receiver)?;

        let statements = new_property_statement(client.package_id(), &mut ptb, wanted_statements)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!("accredit").into(),
            vec![],
            vec![fed_ref, cap, receiver, statements],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    // Revokes Statement by setting the validity to a specific time

    async fn revoke_statement<C>(
        federation_id: ObjectID,
        statement_name: StatementName,
        valid_to_ms: u64,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::RootAuthority, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let statement_names = statement_name.to_ptb(&mut ptb, client.package_id())?;

        let valid_to_ms = ptb.pure(valid_to_ms)?;
        let statement_names = new_statement_name(statement_name, &mut ptb, client.package_id())?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("remove_statement").into(),
            vec![],
            vec![fed_ref, cap, statement_names, valid_to_ms],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a user's attestation accreditation.
    ///
    /// Removes specific attestation permissions from a user. The revoker must have
    /// sufficient permissions to revoke the target accreditation.
    async fn revoke_accreditation_to_attest<C>(
        federation_id: ObjectID,
        entity_id: ObjectID,
        permission_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Attest, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let entity_id = ptb.pure(entity_id)?;
        let permission_id = ptb.pure(permission_id)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!(move_names::PACKAGE_NAME).into(),
            ident_str!("revoke_accreditation_to_attest").into(),
            vec![],
            vec![fed_ref, cap, entity_id, permission_id],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Adds a new root authority to the federation.
    ///
    /// Root authorities have the highest trust level and can perform all operations.
    /// The new authority receives a `RootAuthorityCap`. Requires existing `RootAuthorityCap`.
    async fn add_root_authority<C>(
        federation_id: ObjectID,
        account_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::RootAuthority, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let account_id_arg = ptb.pure(account_id)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::PACKAGE_NAME).into(),
            ident_str!("add_root_authority").into(),
            vec![],
            vec![fed_ref, cap, account_id_arg],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Grants accreditation permissions to another user.
    ///
    /// Allows the receiver to further delegate accreditation rights for the specified statements.
    /// The granter must have sufficient permissions for all statements being delegated.
    async fn create_accreditation_to_accredit<C>(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_statements: Vec<Statement>,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Accredit, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let receiver_arg = ptb.pure(receiver)?;

        let want_statements = new_statement(client.package_id(), &mut ptb, want_statements)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::PACKAGE_NAME).into(),
            ident_str!("create_accreditation_to_accredit").into(),
            vec![],
            vec![fed_ref, cap, receiver_arg, want_statements],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Grants attestation permissions to another user.
    ///
    /// Allows the receiver to create attestations for the specified statements.
    /// The granter must have sufficient permissions for all statements being delegated.
    async fn create_accreditation_to_attest<C>(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_statements: Vec<Statement>,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Attest, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let receiver_arg = ptb.pure(receiver)?;

        let want_statements = new_statement(client.package_id(), &mut ptb, want_statements)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("create_accreditation_to_attest").into(),
            vec![],
            vec![fed_ref, cap, receiver_arg, want_statements],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a user's accreditation permissions.
    ///
    /// Removes specific accreditation rights from a user. The revoker must have
    /// sufficient permissions to revoke the target accreditation.
    async fn revoke_accreditation_to_accredit<C>(
        federation_id: ObjectID,
        entity_id: ObjectID,
        permission_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Accredit, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let entity_id = ptb.pure(entity_id)?;
        let permission_id = ptb.pure(permission_id)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::PACKAGE_NAME).into(),
            ident_str!("revoke_accreditation_to_accredit").into(),
            vec![],
            vec![fed_ref, cap, entity_id, permission_id],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Retrieves all statement names registered in the federation.
    ///
    /// Returns a list of all statement types that can be attested within the federation.
    async fn get_statements<C>(federation_id: ObjectID, client: &C) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("get_statements").into(),
            vec![],
            vec![fed_ref],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Checks if a statement is registered in the federation.
    ///
    /// Verifies whether a specific statement type exists within the federation's
    /// governance structure.
    async fn is_statement_in_federation<C>(
        federation_id: ObjectID,
        statement_name: StatementName,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let statement_name = CallArg::Pure(bcs::to_bytes(&statement_name)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("is_statement_in_federation").into(),
            vec![],
            vec![fed_ref, statement_name],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Retrieves attestation accreditations for a specific user.
    ///
    /// Returns the set of statements a user is authorized to attest, along with
    /// any value constraints.
    async fn get_accreditations_to_attest<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!("get_accreditations_to_attest").into(),
            vec![],
            vec![fed_ref, user_id],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Checks if a user has attestation permissions.
    ///
    /// Returns true if the user has any attestation accreditations in the federation.
    async fn is_attester<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!("is_attester").into(),
            vec![],
            vec![fed_ref, user_id],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Retrieves accreditation permissions for a specific user.
    ///
    /// Returns the set of statements a user is authorized to delegate to others
    /// for accreditation purposes.
    async fn get_accreditations_to_accredit<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!("get_accreditations_to_accredit").into(),
            vec![],
            vec![fed_ref, user_id],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Checks if a user has accreditation delegation permissions.
    ///
    /// Returns true if the user can grant accreditation rights to others.
    async fn is_accreditor<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!("is_accreditor").into(),
            vec![],
            vec![fed_ref, user_id],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a statement by setting its validity expiration time.
    ///
    /// Sets a time limit on when a statement is considered valid. After the specified
    /// time, the statement can no longer be attested. Requires `RootAuthorityCap`.
    async fn revoke_statement<C>(
        federation_id: ObjectID,
        statement_name: StatementName,
        valid_to_ms: u64,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::RootAuthority, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let statement_name = new_statement_name(statement_name, &mut ptb, client.package_id())?;

        let valid_to_ms = ptb.pure(valid_to_ms)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!("revoke_statement").into(),
            vec![],
            vec![fed_ref, cap, statement_name, valid_to_ms],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Validates a single statement against federation rules.
    ///
    /// Checks if the specified attester has permission to attest the given
    /// statement name and value combination according to their accreditations.
    async fn validate_statement<C>(
        federation_id: ObjectID,
        attester_id: ObjectID,
        statement_name: StatementName,
        statement_value: StatementValue,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let attester_id = ptb.pure(attester_id)?;

        let statement_name = statement_name.to_ptb(&mut ptb, client.package_id())?;

        let statement_name_arg = new_statement_name(statement_name, &mut ptb, client.package_id())?;

        let statement_value_arg = match statement_value {
            StatementValue::Text(text) => new_statement_value_string(text, &mut ptb, client.package_id())?,
            StatementValue::Number(number) => new_statement_value_number(number, &mut ptb, client.package_id())?,
        };

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(MAIN_ITH_MODULE).into(),
            ident_str!("validate_statement").into(),
            vec![],
            vec![fed_ref, attester_id, statement_name, statement_value, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Validates multiple statements against federation rules.
    ///
    /// Checks if the specified issuer has permission to attest all provided
    /// statement name-value pairs according to their accreditations.
    async fn validate_statements<C>(
        federation_id: ObjectID,
        entity_id: ObjectID,
        statements: HashMap<StatementName, StatementValue>,
        client: &C,
    ) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let mut statement_names = vec![];
        let mut statement_values = vec![];

        for (statement_name, statement_value) in statements.iter() {
            let names = statement_name.names();
            let name = ptb.pure(names)?;
            let statement_name: Argument = ptb.programmable_move_call(
                client.package_id(),
                ident_str!(move_names::MODULE_NAME).into(),
                ident_str!("new_statement_name_from_vector").into(),
                vec![],
                vec![name],
            );
            statement_names.push(statement_name);

            let statement_value = match statement_value {
                StatementValue::Text(text) => {
                    let v = ptb.pure(text)?;
                    ptb.programmable_move_call(
                        client.package_id(),
                        ident_str!(move_names::MODULE_VALUE).into(),
                        ident_str!("new_statement_value_string").into(),
                        vec![],
                        vec![v],
                    )
                }
                StatementValue::Number(number) => {
                    let v = ptb.pure(number)?;
                    ptb.programmable_move_call(
                        client.package_id(),
                        ident_str!(move_names::MODULE_VALUE).into(),
                        ident_str!("new_statement_value_number").into(),
                        vec![],
                        vec![v],
                    )
                }
            };
            statement_values.push(statement_value);
        }

        let statement_name_tag =
            TypeTag::from_str(format!("{}::{}::StatementName", client.package_id(), move_names::MODULE_NAME).as_str())?;
        let statement_value_tag = TypeTag::from_str(
            format!("{}::{}::StatementValue", client.package_id(), move_names::MODULE_VALUE).as_str(),
        )?;

        let statement_names = ptb.command(Command::MakeMoveVec(Some(statement_name_tag.clone()), statement_names));
        let statement_values = ptb.command(Command::MakeMoveVec(
            Some(statement_value_tag.clone()),
            statement_values,
        ));

        let statements = ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_UTILS).into(),
            ident_str!("vec_map_from_keys_values").into(),
            vec![statement_name_tag, statement_value_tag],
            vec![statement_names, statement_values],
        );

        let entity_id = ptb.pure(entity_id)?;
        let clock = super::get_clock_ref(&mut ptb);

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("validate_statements").into(),
            vec![],
            vec![fed_ref, entity_id, statements, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }
}

impl ITHOperations for ITHImpl {}
