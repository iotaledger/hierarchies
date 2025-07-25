// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # ITH Operations
//!
//! This module provides low-level operations for interacting with the ITH (IOTA Trust Hierarchy) module.
//! It handles capability management, federation references, and transaction building.
//!
//! ## Trust Hierarchy Model
//!
//! ITH operates on a three-tier capability system:
//! - **`RootAuthority`**: Full administrative control over federations
//! - **`Accredit`**: Permission to delegate accreditation rights to others
//! - **`Attest`**: Permission to create attestations for specific statements
//!
//! Capabilities are represented as owned objects in the IOTA network, ensuring
//! secure and verifiable permission management.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use async_trait::async_trait;
use iota_interaction::move_types::language_storage::StructTag;
use iota_interaction::rpc_types::{IotaObjectDataFilter, IotaObjectDataOptions, IotaObjectResponseQuery};
use iota_interaction::types::base_types::{IotaAddress, ObjectID, ObjectRef, SequenceNumber};
use iota_interaction::types::object::Owner;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::{CallArg, Command, ObjectArg, ProgrammableTransaction};
use iota_interaction::{ident_str, IotaClientTrait, MoveType, OptionalSync};
use product_common::core_client::CoreClientReadOnly;

use crate::core::error::{CapabilityError, FederationError, OperationError};
use crate::core::types::statements::name::StatementName;
use crate::core::types::statements::value::StatementValue;
use crate::core::types::statements::{new_property_statement, Statement};
use crate::core::types::Capability;
use crate::error::{NetworkError, ObjectError};
use crate::utils::{self};

const MAIN_ITH_MODULE: &str = move_names::MODULE_MAIN;

/// Move package module names for ITH smart contract interactions.
///
/// These constants define the module names used when calling functions
/// in the ITH Move package deployed on the IOTA network.
pub mod move_names {
    /// The main ITH package name
    pub const PACKAGE_NAME: &str = "ith";
    /// Main module containing federation and core operations
    pub const MODULE_MAIN: &str = "main";
    /// Module for statement-related operations
    pub const MODULE_STATEMENT: &str = "statement";
    /// Module for statement value operations
    pub const MODULE_VALUE: &str = "statement_value";
    /// Module for statement name operations
    pub const MODULE_NAME: &str = "statement_name";
    /// Module for statement condition operations
    pub const MODULE_CONDITION: &str = "statement_condition";
    /// Utility module for common operations
    pub const MODULE_UTILS: &str = "utils";
}

/// Internal implementation of ITH operations.
///
/// This struct provides low-level operations for interacting with the ITH (IOTA Trust Hierarchy) module.
/// It handles capability management, federation references, and transaction building.
///
/// ## Trust Hierarchy Model
///
/// ITH operates on a three-tier capability system:
/// - **`RootAuthority`**: Full administrative control over federations
/// - **`Accredit`**: Permission to delegate accreditation rights to others
/// - **`Attest`**: Permission to create attestations for specific statements
///
/// Capabilities are represented as owned objects in the IOTA network, ensuring
/// secure and verifiable permission management.
#[derive(Debug, Clone)]
pub(crate) struct ITHImpl;

impl ITHImpl {
    /// Retrieves a capability object for the specified sender.
    ///
    /// Capabilities grant permissions within the ITH system:
    /// - `RootAuthority`: Full administrative access
    /// - `Accredit`: Permission to delegate accreditation rights
    /// - `Attest`: Permission to create attestations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The sender doesn't own the requested capability type
    /// - The capability object structure is invalid
    /// - Network communication fails
    pub(crate) async fn get_cap<C>(
        client: &C,
        cap_type: Capability,
        sender: IotaAddress,
    ) -> Result<ObjectRef, CapabilityError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let cap_tag =
            StructTag::from_str(&format!("{}::{MAIN_ITH_MODULE}::{cap_type}", client.package_id())).map_err(|_| {
                CapabilityError::InvalidType {
                    cap_type: cap_type.to_string(),
                }
            })?;

        let filter = IotaObjectResponseQuery::new_with_filter(IotaObjectDataFilter::StructType(cap_tag));

        let mut cursor = None;
        loop {
            let mut page = client
                .client_adapter()
                .read_api()
                .get_owned_objects(sender, Some(filter.clone()), cursor, None)
                .await
                .map_err(|_e| CapabilityError::NotFound {
                    cap_type: cap_type.to_string(),
                })?;

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

        Err(CapabilityError::NotFound {
            cap_type: cap_type.to_string(),
        })
    }

    /// Creates a shared object reference for a federation.
    ///
    /// Federations are shared objects in the ITH system, requiring proper
    /// reference handling for transaction building. This function retrieves
    /// the initial shared version needed for transaction construction.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn get_fed_ref<C>(client: &C, federation_id: ObjectID) -> Result<ObjectArg, FederationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let fed_ref = ObjectArg::SharedObject {
            id: federation_id,
            initial_shared_version: ITHImpl::initial_shared_version(client, &federation_id)
                .await
                .map_err(FederationError::Object)?,
            mutable: true,
        };

        Ok(fed_ref)
    }

    /// Retrieves the initial shared version of a shared object.
    ///
    /// Required for properly referencing shared objects in IOTA transactions.
    /// Returns an error if the object is not shared.
    pub(crate) async fn initial_shared_version<C>(
        client: &C,
        object_id: &ObjectID,
    ) -> Result<SequenceNumber, ObjectError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let owner = client
            .client_adapter()
            .read_api()
            .get_object_with_options(*object_id, IotaObjectDataOptions::default().with_owner())
            .await
            .map_err(|e| ObjectError::RetrievalFailed {
                source: Box::new(NetworkError::RpcFailed { source: Box::new(e) }),
            })?
            .owner()
            .ok_or_else(|| ObjectError::NotFound {
                id: object_id.to_string(),
            })?;

        match owner {
            Owner::Shared { initial_shared_version } => Ok(initial_shared_version),
            _ => Err(ObjectError::WrongType {
                expected: "SharedObject".to_string(),
                actual: "ImmOrOwnedObject".to_string(),
            }),
        }
    }
}

/// High-level ITH operations trait.
///
/// Provides methods for managing federations, statements, and accreditations.
/// Each method returns a `ProgrammableTransaction` that can be executed on-chain.
///
/// ## Core Operations
///
/// - **Federation Management**: Create and configure trust federations
/// - **Statement Management**: Define and manage attestable statement types
/// - **Accreditation**: Grant and revoke permission to delegate trust
/// - **Attestation**: Grant and revoke permission to create attestations
/// - **Validation**: Verify entity permissions for specific statements
///
/// All operations require appropriate capabilities and return transactions
/// ready for execution on the IOTA network.
#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
pub(crate) trait ITHOperations {
    /// Creates a new federation with the caller as the initial root authority.
    ///
    /// The federation is a shared object that manages trust hierarchies.
    /// The creator receives all three capability types: `RootAuthorityCap`,
    /// `AccreditCap`, and `AttestCap`, granting full control over the federation.
    ///
    /// # Parameters
    ///
    /// - `package_id`: The ITH Move package ID deployed on the network
    ///
    /// # Returns
    ///
    /// [`ProgrammableTransaction`] A transaction that when executed creates a
    /// new federation and grants
    /// the sender all initial capabilities.
    fn new_federation(package_id: ObjectID) -> Result<ProgrammableTransaction, OperationError> {
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
    /// You can either restrict allowed values to a specific set or allow any values.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The target federation
    /// - `statement_name`: Unique identifier for the statement type
    /// - `allowed_values`: Specific values permitted for this statement (ignored if `allow_any` is true)
    /// - `allow_any`: Whether to allow any values for this statement type
    /// - `owner`: Address that owns the required `RootAuthorityCap`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The owner doesn't have `RootAuthorityCap`
    /// - The statement name already exists in the federation
    /// - Network or transaction building fails
    async fn add_statement<C>(
        federation_id: ObjectID,
        statement_name: StatementName,
        allowed_values: HashSet<StatementValue>,
        allow_any: bool,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::RootAuthority, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let allow_any = ptb.pure(allow_any)?;

        let statement_names = statement_name.to_ptb(&mut ptb, client.package_id())?;

        let value_tag = StatementValue::move_type(client.package_id());

        let mut values_of_property = vec![];
        for property_value in allowed_values {
            let value = property_value.to_ptb(&mut ptb, client.package_id())?;
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

    /// Revokes a user's attestation accreditation.
    ///
    /// Removes specific attestation permissions from a user. The revoker must have
    /// sufficient permissions to revoke the target accreditation.
    async fn revoke_accreditation_to_attest<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        permission_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Attest, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let user_id_arg = ptb.pure(user_id)?;
        let permission_id = ptb.pure(permission_id)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_accreditation_to_attest").into(),
            vec![],
            vec![fed_ref, cap, user_id_arg, permission_id],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Adds a new root authority to the federation.
    ///
    /// Root authorities have the highest trust level and can perform all
    /// operations.
    /// The new authority receives a `RootAuthorityCap`. Requires existing
    /// `RootAuthorityCap`.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The target federation
    /// - `account_id`: The new root authority's account ID
    /// - `owner`: Address that owns the required `RootAuthorityCap`
    /// - `client`: The client to use for the operation
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `RootAuthorityCap`.
    async fn add_root_authority<C>(
        federation_id: ObjectID,
        account_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
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
            ident_str!(move_names::MODULE_MAIN).into(),
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
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The target federation
    /// - `receiver`: The user receiving the accreditation
    /// - `want_statements`: The statements the receiver wants to delegate
    /// - `owner`: Address that owns the required `AccreditCap`
    /// - `client`: The client to use for the operation
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `AccreditCap`.
    async fn create_accreditation_to_accredit<C>(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_statements: Vec<Statement>,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Accredit, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let receiver_arg = ptb.pure(receiver)?;

        let want_statements = new_property_statement(client.package_id(), &mut ptb, want_statements)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
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
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The target federation
    /// - `receiver`: The user receiving the attestation accreditation
    /// - `want_statements`: The statements the receiver wants to attest
    /// - `owner`: Address that owns the required `AttestCap`
    /// - `client`: The client to use for the operation
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `AttestCap`.
    async fn create_accreditation_to_attest<C>(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_statements: Vec<Statement>,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Attest, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let receiver_arg = ptb.pure(receiver)?;

        let want_statements = new_property_statement(client.package_id(), &mut ptb, want_statements)?;

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
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The target federation
    /// - `user_id`: The user whose accreditation permissions to revoke
    /// - `permission_id`: The specific accreditation permission to revoke
    /// - `owner`: Address that owns the required `AccreditCap`
    /// - `client`: The client to use for the operation
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `AccreditCap`.
    async fn revoke_accreditation_to_accredit<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        permission_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::Accredit, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let user_id_arg = ptb.pure(user_id)?;
        let permission_id = ptb.pure(permission_id)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_accreditation_to_accredit").into(),
            vec![],
            vec![fed_ref, cap, user_id_arg, permission_id],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Retrieves all statement names registered in the federation.
    ///
    /// Returns a list of all statement types that can be attested within the federation.
    /// This includes both active and revoked statements, but validation functions
    /// will check expiration times when validating attestations.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The federation to query
    /// - `client`: The client to use for the operation
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn get_statements<C>(federation_id: ObjectID, client: &C) -> Result<ProgrammableTransaction, OperationError>
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
    /// governance structure. This check is independent of the statement's
    /// revocation status.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The federation to query
    /// - `statement_name`: The statement type to check for
    /// - `client`: The client to use for the operation
    ///
    /// # Returns
    ///
    /// A transaction that when executed returns true if the statement exists
    /// in the federation, false otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn is_statement_in_federation<C>(
        federation_id: ObjectID,
        statement_name: StatementName,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
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
    /// any value constraints. This shows what statements the user can create
    /// attestations for, but not what they can delegate to others.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The federation to query
    /// - `user_id`: The user whose attestation permissions to check
    /// - `client`: The client to use for the operation
    ///
    /// # Returns
    ///
    /// A transaction that when executed returns the user's attestation
    /// accreditations and their associated constraints.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn get_accreditations_to_attest<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
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
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
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
    /// for accreditation purposes. This shows what statements the user can
    /// grant others permission to further delegate (create_accreditation_to_accredit).
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The federation to query
    /// - `user_id`: The user whose accreditation permissions to check
    ///
    /// # Returns
    ///
    /// A transaction that when executed returns the user's accreditation
    /// permissions and their associated constraints.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn get_accreditations_to_accredit<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
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
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("is_accreditor").into(),
            vec![],
            vec![fed_ref, user_id],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a statement immediately using the current timestamp.
    ///
    /// Sets the statement's validity expiration to the current time, effectively
    /// revoking it immediately. After revocation, the statement can no longer be
    /// attested. Requires `RootAuthorityCap`.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The federation containing the statement
    /// - `statement_name`: The statement type to revoke
    /// - `owner`: Address that owns the required `RootAuthorityCap`
    /// - `client`: The client to use for the operation
    ///
    /// # Returns
    ///
    /// A transaction that when executed revokes the statement.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `RootAuthorityCap` or the
    /// statement doesn't exist in the federation.
    async fn revoke_statement<C>(
        federation_id: ObjectID,
        statement_name: StatementName,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::RootAuthority, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let statement_name = statement_name.to_ptb(&mut ptb, client.package_id())?;

        let clock = super::get_clock_ref(&mut ptb);

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_statement").into(),
            vec![],
            vec![fed_ref, cap, statement_name, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a statement at a specific future timestamp.
    ///
    /// Sets a specific time limit on when a statement is considered valid. After
    /// the specified time, the statement can no longer be attested. This allows
    /// for scheduled revocation. Requires `RootAuthorityCap`.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The federation containing the statement
    /// - `statement_name`: The statement type to revoke
    /// - `valid_to_ms`: Timestamp in milliseconds when the statement expires
    /// - `owner`: Address that owns the required `RootAuthorityCap`
    /// - `client`: The client to use for the operation
    ///
    /// # Returns
    ///
    /// A transaction that when executed revokes the statement.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `RootAuthorityCap` or the
    /// statement doesn't exist in the federation.
    async fn revoke_statement_at<C>(
        federation_id: ObjectID,
        statement_name: StatementName,
        valid_to_ms: u64,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ITHImpl::get_cap(client, Capability::RootAuthority, owner).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let statement_name = statement_name.to_ptb(&mut ptb, client.package_id())?;

        let valid_to_ms = ptb.pure(valid_to_ms)?;
        let clock = super::get_clock_ref(&mut ptb);

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_statement_at").into(),
            vec![],
            vec![fed_ref, cap, statement_name, valid_to_ms, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Validates a single statement against federation rules.
    ///
    /// Checks if the specified attester has permission to attest the given
    /// statement name and value combination according to their accreditations.
    /// This creates a transaction that will return true if the attester can
    /// make the attestation, false otherwise.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The federation containing the statement rules
    /// - `attester_id`: The entity whose attestation permissions to check
    /// - `statement_name`: The statement type to validate
    /// - `statement_value`: The specific value being attested
    /// - `client`: The client to use for the operation
    ///
    /// # Returns
    ///
    /// A transaction that when executed returns a boolean indicating whether
    /// the attestation is valid according to federation rules.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn validate_statement<C>(
        federation_id: ObjectID,
        attester_id: ObjectID,
        statement_name: StatementName,
        statement_value: StatementValue,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let attester_id = ptb.pure(attester_id)?;

        let statement_name = statement_name.to_ptb(&mut ptb, client.package_id())?;

        let statement_value = statement_value.to_ptb(&mut ptb, client.package_id())?;

        let clock = super::get_clock_ref(&mut ptb);

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("validate_statement").into(),
            vec![],
            vec![fed_ref, attester_id, statement_name, statement_value, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Validates multiple statements against federation rules.
    ///
    /// Checks if the specified entity has permission to attest all provided
    /// statement name-value pairs according to their accreditations.
    /// This is more efficient than multiple single-statement validations.
    ///
    /// # Parameters
    ///
    /// - `federation_id`: The federation containing the statement rules
    /// - `entity_id`: The entity whose attestation permissions to check
    /// - `statements`: Map of statement names to their values for validation
    /// - `client`: The client to use for the operation
    ///
    /// # Returns
    ///
    /// A transaction that when executed returns true only if the entity
    /// has permission to attest all provided statements.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn validate_statements<C>(
        federation_id: ObjectID,
        entity_id: ObjectID,
        statements: HashMap<StatementName, StatementValue>,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = ITHImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let mut statement_names = vec![];
        let mut statement_values = vec![];

        for (statement_name, statement_value) in statements.iter() {
            let statement_name = statement_name.to_ptb(&mut ptb, client.package_id())?;
            statement_names.push(statement_name);

            let statement_value = statement_value.to_ptb(&mut ptb, client.package_id())?;
            statement_values.push(statement_value);
        }

        let statement_name_tag = StatementName::move_type(client.package_id());
        let statement_value_tag = StatementValue::move_type(client.package_id());

        let statement_names = ptb.command(Command::MakeMoveVec(Some(statement_name_tag.clone()), statement_names));
        let statement_values = ptb.command(Command::MakeMoveVec(
            Some(statement_value_tag.clone()),
            statement_values,
        ));

        let statements = ptb.programmable_move_call(
            client.package_id(),
            ident_str!("utils").into(),
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
