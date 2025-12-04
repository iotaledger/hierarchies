// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Operations
//!
//! This module provides low-level operations for interacting with the Hierarchies (IOTA Trust Hierarchy) module.
//! It handles capability management, federation references, and transaction building.
//!
//! ## Trust Hierarchy Model
//!
//! Hierarchies operates on a two-tier capability system:
//! - **`RootAuthority`**: Full administrative control over federations
//! - **`Accredit`**: Permission to delegate both accreditation and attestation rights to others
//!
//! Capabilities are represented as owned objects in the IOTA network, ensuring
//! secure and verifiable permission management.

use std::collections::HashMap;

use async_trait::async_trait;
use iota_interaction::rpc_types::IotaObjectDataOptions;
use iota_interaction::types::base_types::{IotaAddress, ObjectID, ObjectRef, SequenceNumber};
use iota_interaction::types::object::Owner;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::{CallArg, Command, ObjectArg, ProgrammableTransaction};
use iota_interaction::{IotaClientTrait, MoveType, OptionalSync, ident_str};
use product_common::core_client::CoreClientReadOnly;

use crate::core::error::OperationError;
use crate::core::types::property::{FederationProperty, new_properties, new_property};
use crate::core::types::property_name::PropertyName;
use crate::core::types::property_value::PropertyValue;
use crate::core::types::{ACCREDIT_CAP_TYPE, AccreditCap, ROOT_AUTHORITY_CAP_TYPE, RootAuthorityCap, move_names};
use crate::core::{CapabilityError, get_clock_ref};
use crate::error::{NetworkError, ObjectError};

/// Internal implementation of Hierarchies operations.
///
/// This struct provides low-level operations for interacting with the Hierarchies (IOTA Trust Hierarchy) module.
/// It handles capability management, federation references, and transaction building.
///
/// ## Trust Hierarchy Model
///
/// Hierarchies operates on a two-tier capability system:
/// - **`RootAuthority`**: Full administrative control over federations
/// - **`Accredit`**: Permission to delegate both accreditation and attestation rights to others
///
/// Capabilities are represented as owned objects in the IOTA network, ensuring
/// secure and verifiable permission management.
#[derive(Debug, Clone)]
pub(crate) struct HierarchiesImpl;

impl HierarchiesOperations for HierarchiesImpl {}

impl HierarchiesImpl {
    /// Retrieves a RootAuthorityCap for the specified owner.
    ///
    /// This method searches across all package versions in history to find
    /// a capability object owned by the sender, which is necessary after package upgrades.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have a RootAuthorityCap.
    pub(crate) async fn get_root_authority_cap<C>(
        client: &C,
        owner: IotaAddress,
        federation_id: ObjectID,
    ) -> Result<ObjectRef, CapabilityError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let cap: RootAuthorityCap = client
            .find_object_for_address(owner, |cap: &RootAuthorityCap| cap.federation_id == federation_id)
            .await
            .map_err(|e| CapabilityError::Generic {
                message: "Failed to find object for address".to_string(),
                source: e.into(),
            })?
            .ok_or_else(|| CapabilityError::NotFound {
                cap_type: ROOT_AUTHORITY_CAP_TYPE.to_string(),
            })?;

        let object_id = *cap.id.object_id();

        client
            .get_object_ref_by_id(object_id)
            .await
            .map_err(|e| CapabilityError::Generic {
                message: "Failed to get object ref by id".to_string(),
                source: e.into(),
            })?
            .map(|owned_ref| owned_ref.reference.to_object_ref())
            .ok_or_else(|| CapabilityError::NotFound {
                cap_type: ROOT_AUTHORITY_CAP_TYPE.to_string(),
            })
    }

    /// Retrieves an AccreditCap for the specified owner.
    ///
    /// This method searches across all package versions in history to find
    /// a capability object owned by the sender, which is necessary after package upgrades.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have an AccreditCap.
    pub(crate) async fn get_accredit_cap<C>(
        client: &C,
        owner: IotaAddress,
        federation_id: ObjectID,
    ) -> Result<ObjectRef, CapabilityError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let cap: AccreditCap = client
            .find_object_for_address(owner, |cap: &AccreditCap| cap.federation_id == federation_id)
            .await
            .map_err(|e| CapabilityError::Generic {
                message: "Failed to find object for address".to_string(),
                source: e.into(),
            })?
            .ok_or_else(|| CapabilityError::NotFound {
                cap_type: ACCREDIT_CAP_TYPE.to_string(),
            })?;

        let object_id = *cap.id.object_id();
        client
            .get_object_ref_by_id(object_id)
            .await
            .map_err(|e| CapabilityError::Generic {
                message: "Failed to get object ref by id".to_string(),
                source: e.into(),
            })?
            .map(|owned_ref| owned_ref.reference.to_object_ref())
            .ok_or_else(|| CapabilityError::NotFound {
                cap_type: ACCREDIT_CAP_TYPE.to_string(),
            })
    }

    /// Creates a shared object reference for a federation.
    ///
    /// Federations are shared objects in the Hierarchies system, requiring proper
    /// reference handling for transaction building. This function retrieves
    /// the initial shared version needed for transaction construction.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn get_fed_ref<C>(client: &C, federation_id: ObjectID) -> Result<ObjectArg, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let fed_ref = ObjectArg::SharedObject {
            id: federation_id,
            initial_shared_version: HierarchiesImpl::initial_shared_version(client, &federation_id)
                .await
                .map_err(|e| OperationError::Object(ObjectError::RetrievalFailed { source: Box::new(e) }))?,
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

/// High-level Hierarchies operations trait.
///
/// Provides methods for managing federations, properties, and accreditations.
/// Each method returns a `ProgrammableTransaction` that can be executed on-chain.
///
/// ## Core Operations
///
/// - **Federation Management**: Create and configure trust federations
/// - **Property Management**: Define and manage attestable property types
/// - **Accreditation**: Grant and revoke permission to delegate trust
/// - **Attestation**: Grant and revoke permission to create attestations
/// - **Validation**: Verify entity permissions for specific properties
///
/// All operations require appropriate capabilities and return transactions
/// ready for execution on the IOTA network.
#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
pub(crate) trait HierarchiesOperations {
    /// Creates a new federation with the caller as the initial root authority.
    ///
    /// The federation is a shared object that manages trust hierarchies.
    /// The creator receives two capability types: `RootAuthorityCap`
    /// and `AccreditCap`, granting full control over the federation.
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

    /// Adds a new property type to the federation.
    ///
    /// Properties define the types of claims that can be attested within the federation.
    /// You can either restrict allowed values to a specific set or allow any values.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The owner doesn't have `RootAuthorityCap`
    /// - The property name already exists in the federation
    /// - Network or transaction building fails
    async fn add_property<C>(
        federation_id: ObjectID,
        property: FederationProperty,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = HierarchiesImpl::get_root_authority_cap(client, owner, federation_id).await?;
        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;
        let property = new_property(client.package_id(), &mut ptb, property)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("add_property").into(),
            vec![],
            vec![fed_ref, cap, property],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a user's attestation accreditation.
    ///
    /// This function revokes specific attestation accreditations from a user.
    /// The revoker must possess sufficient accreditation to revoke the target accreditation.
    async fn revoke_accreditation_to_attest<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        accreditation_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = HierarchiesImpl::get_accredit_cap(client, owner, federation_id).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let user_id_arg = ptb.pure(user_id)?;
        let permission_id = ptb.pure(accreditation_id)?;
        let clock = get_clock_ref(&mut ptb);
        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_accreditation_to_attest").into(),
            vec![],
            vec![fed_ref, cap, user_id_arg, permission_id, clock],
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

        let cap = HierarchiesImpl::get_root_authority_cap(client, owner, federation_id).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
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
    /// Allows the receiver to further delegate accreditation rights for the specified properties.
    /// The granter must have sufficient permissions for all properties being delegated.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `AccreditCap`.
    async fn create_accreditation_to_accredit<C>(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_properties: Vec<FederationProperty>,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = HierarchiesImpl::get_accredit_cap(client, owner, federation_id).await?;
        let clock = get_clock_ref(&mut ptb);

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let receiver_arg = ptb.pure(receiver)?;

        let want_properties = new_properties(client.package_id(), &mut ptb, want_properties)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("create_accreditation_to_accredit").into(),
            vec![],
            vec![fed_ref, cap, receiver_arg, want_properties, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Grants attestation permissions to another user.
    ///
    /// Allows the receiver to create attestations for the specified properties.
    /// The granter must have sufficient permissions for all properties being delegated.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `AccreditCap`.
    async fn create_accreditation_to_attest<C>(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_properties: Vec<FederationProperty>,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = HierarchiesImpl::get_accredit_cap(client, owner, federation_id).await?;
        let clock = get_clock_ref(&mut ptb);
        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let receiver_arg = ptb.pure(receiver)?;

        let want_properties = new_properties(client.package_id(), &mut ptb, want_properties)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("create_accreditation_to_attest").into(),
            vec![],
            vec![fed_ref, cap, receiver_arg, want_properties, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a user's accreditation permissions.
    ///
    /// Removes specific accreditation rights from a user. The revoker must have
    /// sufficient permissions to revoke the target accreditation.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `AccreditCap`.
    async fn revoke_accreditation_to_accredit<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        accreditation_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = HierarchiesImpl::get_accredit_cap(client, owner, federation_id).await?;
        let clock = get_clock_ref(&mut ptb);
        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let user_id_arg = ptb.pure(user_id)?;
        let accreditation_id = ptb.pure(accreditation_id)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_accreditation_to_accredit").into(),
            vec![],
            vec![fed_ref, cap, user_id_arg, accreditation_id, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Retrieves all property names registered in the federation.
    ///
    /// Returns a list of all property types that can be attested within the federation.
    /// This includes both active and revoked properties, but validation functions
    /// will check expiration times when validating properties.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn get_properties<C>(federation_id: ObjectID, client: &C) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("get_properties").into(),
            vec![],
            vec![fed_ref],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Checks if a property is registered in the federation.
    ///
    /// Verifies whether a specific property type exists within the federation's
    /// governance structure. This check is independent of the property's
    /// revocation status.
    ///
    ///
    /// # Returns
    ///
    /// A transaction that when executed returns true if the property exists
    /// in the federation, false otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn is_property_in_federation<C>(
        federation_id: ObjectID,
        property_name: PropertyName,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let property_name = CallArg::Pure(bcs::to_bytes(&property_name)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("is_property_in_federation").into(),
            vec![],
            vec![fed_ref, property_name],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Retrieves attestation accreditations for a specific user.
    ///
    /// Returns the set of properties a user is authorized to attest, along with
    /// any value constraints. This shows what properties the user can create
    /// attestations for, but not what they can delegate to others.
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

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
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

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
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
    /// Returns the set of properties a user is authorized to delegate to others
    /// for accreditation purposes. This shows what properties the user can
    /// grant others permission to further delegate (create_accreditation_to_accredit).
    ///
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

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
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

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
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

    /// Revokes a property immediately using the current timestamp.
    ///
    /// Sets the property's validity expiration to the current time, effectively
    /// revoking it immediately. After revocation, the property can no longer be
    /// attested. Requires `RootAuthorityCap`.
    ///
    /// # Returns
    ///
    /// A transaction that when executed revokes the property.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `RootAuthorityCap` or the
    /// property doesn't exist in the federation.
    async fn revoke_property<C>(
        federation_id: ObjectID,
        property_name: PropertyName,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = HierarchiesImpl::get_root_authority_cap(client, owner, federation_id).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let property_name = property_name.to_ptb(&mut ptb, client.package_id())?;

        let clock = get_clock_ref(&mut ptb);

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_property").into(),
            vec![],
            vec![fed_ref, cap, property_name, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a property at a specific future timestamp.
    ///
    /// Sets a specific time limit on when a property is considered valid. After
    /// the specified time, the property can no longer be attested. This allows
    /// for scheduled revocation. Requires `RootAuthorityCap`.
    ///
    ///
    /// # Returns
    ///
    /// A transaction that when executed revokes the property.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner doesn't have `RootAuthorityCap` or the
    /// property doesn't exist in the federation.
    async fn revoke_property_at<C>(
        federation_id: ObjectID,
        property_name: PropertyName,
        valid_to_ms: u64,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = HierarchiesImpl::get_root_authority_cap(client, owner, federation_id).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let property_name = property_name.to_ptb(&mut ptb, client.package_id())?;

        let valid_to_ms = ptb.pure(valid_to_ms)?;
        let clock = get_clock_ref(&mut ptb);

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_property_at").into(),
            vec![],
            vec![fed_ref, cap, property_name, valid_to_ms, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Validates a single property against federation rules.
    ///
    /// Checks if the specified attester has permission to attest the given
    /// property name and value combination according to their accreditations.
    /// This creates a transaction that will return true if the attester can make
    /// the attestation, false otherwise.
    ///
    /// # Returns
    ///
    /// A transaction that when executed returns a boolean indicating whether
    /// the attestation is valid according to federation rules.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn validate_property<C>(
        federation_id: ObjectID,
        attester_id: ObjectID,
        property_name: PropertyName,
        property_value: PropertyValue,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let attester_id = ptb.pure(attester_id)?;

        let property_name = property_name.to_ptb(&mut ptb, client.package_id())?;

        let property_value = property_value.to_ptb(&mut ptb, client.package_id())?;

        let clock = get_clock_ref(&mut ptb);

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("validate_property").into(),
            vec![],
            vec![fed_ref, attester_id, property_name, property_value, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Validates multiple properties against federation rules.
    ///
    /// Checks if the specified entity has permission to attest all provided
    /// property name-value pairs according to their accreditations.
    /// This is more efficient than multiple single-property validations.
    ///
    ///
    /// # Returns
    ///
    /// A transaction that when executed returns true only if the entity
    /// has permission to attest all provided properties.
    ///
    /// # Errors
    ///
    /// Returns an error if the federation object is not found or not shared.
    async fn validate_properties<C>(
        federation_id: ObjectID,
        entity_id: ObjectID,
        properties: HashMap<PropertyName, PropertyValue>,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let mut property_names = vec![];
        let mut property_values = vec![];

        for (property_name, property_value) in properties.iter() {
            let property_name = property_name.to_ptb(&mut ptb, client.package_id())?;
            property_names.push(property_name);

            let property_value = property_value.to_ptb(&mut ptb, client.package_id())?;
            property_values.push(property_value);
        }

        let property_name_tag = PropertyName::move_type(client.package_id());
        let property_value_tag = PropertyValue::move_type(client.package_id());

        let property_names_args = ptb.command(Command::MakeMoveVec(
            Some(property_name_tag.clone().into()),
            property_names,
        ));
        let property_values_args = ptb.command(Command::MakeMoveVec(
            Some(property_value_tag.clone().into()),
            property_values,
        ));

        let properties = ptb.programmable_move_call(
            client.package_id(),
            ident_str!("utils").into(),
            ident_str!("vec_map_from_keys_values").into(),
            vec![property_name_tag, property_value_tag],
            vec![property_names_args, property_values_args],
        );

        let entity_id = ptb.pure(entity_id)?;
        let clock = get_clock_ref(&mut ptb);

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("validate_properties").into(),
            vec![],
            vec![fed_ref, entity_id, properties, clock],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Check if root authority is in the federation.
    async fn is_root_authority<C>(
        federation_id: ObjectID,
        user_id: ObjectID,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = CallArg::Object(fed_ref);
        let user_id = CallArg::Pure(bcs::to_bytes(&user_id)?);

        ptb.move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("is_root_authority").into(),
            vec![],
            vec![fed_ref, user_id],
        )?;

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Revokes a root authority from the federation.
    ///
    /// Only existing root authorities can revoke other root authorities.
    /// Cannot revoke the last root authority to prevent lockout.
    /// The revoked authority's capability remains but becomes unusable.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The owner doesn't have `RootAuthorityCap`
    /// - The account_id is not a root authority
    /// - Attempting to revoke the last root authority
    async fn revoke_root_authority<C>(
        federation_id: ObjectID,
        account_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = HierarchiesImpl::get_root_authority_cap(client, owner, federation_id).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let account_id_arg = ptb.pure(account_id)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("revoke_root_authority").into(),
            vec![],
            vec![fed_ref, cap, account_id_arg],
        );

        let tx = ptb.finish();

        Ok(tx)
    }

    /// Reinstates a previously revoked root authority to the federation.
    ///
    /// This operation allows an existing root authority to restore a revoked root authority
    /// back to active status. The target account must be in the federation's revoked list
    /// and cannot already be an active root authority.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The owner doesn't have `RootAuthorityCap`
    /// - The account is not in the revoked root authorities list
    /// - The account is already an active root authority
    /// - Network communication fails
    async fn reinstate_root_authority<C>(
        federation_id: ObjectID,
        account_id: ObjectID,
        owner: IotaAddress,
        client: &C,
    ) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let mut ptb = ProgrammableTransactionBuilder::new();
        let cap = HierarchiesImpl::get_root_authority_cap(client, owner, federation_id).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

        let fed_ref = HierarchiesImpl::get_fed_ref(client, federation_id).await?;
        let fed_ref = ptb.obj(fed_ref)?;

        let account_id_arg = ptb.pure(account_id)?;

        ptb.programmable_move_call(
            client.package_id(),
            ident_str!(move_names::MODULE_MAIN).into(),
            ident_str!("reinstate_root_authority").into(),
            vec![],
            vec![fed_ref, cap, account_id_arg],
        );

        let tx = ptb.finish();

        Ok(tx)
    }
}
