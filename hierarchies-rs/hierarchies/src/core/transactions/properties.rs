// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Property Management Transactions
//!
//! This module provides transaction implementations for managing properties
//! within Hierarchies federations. Properties define the types of claims that can
//! be attested within a federation.

use std::collections::HashSet;

use async_trait::async_trait;
use iota_interaction::OptionalSync;
use iota_interaction::rpc_types::IotaTransactionBlockEffects;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::ProgrammableTransaction;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::OperationError;
use crate::core::operations::{HierarchiesImpl, HierarchiesOperations};
use crate::core::types::property_name::PropertyName;
use crate::core::types::property_value::PropertyValue;

/// Transaction for adding new property types to federations.
pub mod add_property {
    use super::*;

    /// A transaction that adds a new property type to a federation.
    ///
    /// This transaction allows root authorities to define new types of properties
    /// that can be attested within their federation. You can either restrict
    /// the allowed values to a specific set or allow any values.
    ///
    /// ## Requirements
    ///
    /// - The owner must possess `RootAuthorityCap` for the federation
    /// - The property name must be unique within the federation
    #[derive(Debug, Clone)]
    pub struct AddProperty {
        federation_id: ObjectID,
        name: PropertyName,
        allowed_values: HashSet<PropertyValue>,
        allow_any: bool,
        owner: IotaAddress,
        cached_ptb: OnceCell<ProgrammableTransaction>,
    }

    impl AddProperty {
        /// Creates a new [`AddProperty`] instance.
        ///
        /// # Returns
        ///
        /// A new `AddProperty` transaction instance ready for execution.
        pub fn new(
            federation_id: ObjectID,
            property_name: PropertyName,
            allowed_values: HashSet<PropertyValue>,
            allow_any: bool,
            owner: IotaAddress,
        ) -> Self {
            Self {
                federation_id,
                name: property_name,
                allowed_values,
                allow_any,
                owner,
                cached_ptb: OnceCell::new(),
            }
        }

        /// Builds the programmable transaction for adding a property.
        ///
        /// This method creates the underlying Move transaction that will add
        /// the new property type to the federation with the specified constraints.
        ///
        /// # Returns
        ///
        /// A `ProgrammableTransaction` ready for execution on the IOTA network.
        ///
        /// # Errors
        ///
        /// Returns an error if the owner doesn't have `RootAuthorityCap` or if
        /// the property name already exists in the federation.
        async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, OperationError>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            let ptb = HierarchiesImpl::add_property(
                self.federation_id,
                self.name.clone(),
                self.allowed_values.clone(),
                self.allow_any,
                self.owner,
                client,
            )
            .await?;

            Ok(ptb)
        }
    }

    #[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
    #[cfg_attr(feature = "send-sync", async_trait)]
    impl Transaction for AddProperty {
        type Error = OperationError;

        type Output = ();

        async fn build_programmable_transaction<C>(&self, client: &C) -> Result<ProgrammableTransaction, Self::Error>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            self.cached_ptb.get_or_try_init(|| self.make_ptb(client)).await.cloned()
        }

        async fn apply<C>(mut self, _: &mut IotaTransactionBlockEffects, _: &C) -> Result<Self::Output, Self::Error>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            Ok(())
        }
    }
}

/// Transaction for revoking property types from federations.
pub mod revoke_property {
    use super::*;

    /// A transaction that revokes a property type from a federation.
    ///
    /// This transaction allows root authorities to revoke property types,
    /// preventing future attestations of that type. You can either revoke
    /// immediately or schedule the revocation for a specific future time.
    ///
    /// ## Requirements
    ///
    /// - The owner must possess `RootAuthorityCap` for the federation
    /// - The property must exist in the federation
    #[derive(Debug, Clone)]
    pub struct RevokeProperty {
        federation_id: ObjectID,
        property_name: PropertyName,
        valid_to_ms: Option<u64>,
        owner: IotaAddress,
        cached_ptb: OnceCell<ProgrammableTransaction>,
    }

    impl RevokeProperty {
        /// Creates a new [`RevokeProperty`] instance.
        ///
        /// # Returns
        ///
        /// A new `RevokeProperty` transaction instance ready for execution.
        pub fn new(
            federation_id: ObjectID,
            property_name: PropertyName,
            valid_to_ms: Option<u64>,
            owner: IotaAddress,
        ) -> Self {
            Self {
                federation_id,
                property_name,
                valid_to_ms,
                owner,
                cached_ptb: OnceCell::new(),
            }
        }

        /// Builds the programmable transaction for revoking a property.
        ///
        /// This method creates the underlying Move transaction that will revoke
        /// the property type, either immediately or at a scheduled time.
        ///
        /// # Returns
        ///
        /// A `ProgrammableTransaction` ready for execution on the IOTA network.
        ///
        /// # Errors
        ///
        /// Returns an error if the owner doesn't have `RootAuthorityCap` or if
        /// the property doesn't exist in the federation.
        async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, OperationError>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            let ptb = match self.valid_to_ms {
                Some(valid_to_ms) => {
                    HierarchiesImpl::revoke_property_at(
                        self.federation_id,
                        self.property_name.clone(),
                        valid_to_ms,
                        self.owner,
                        client,
                    )
                    .await?
                }
                None => {
                    HierarchiesImpl::revoke_property(self.federation_id, self.property_name.clone(), self.owner, client)
                        .await?
                }
            };

            Ok(ptb)
        }
    }

    #[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
    #[cfg_attr(feature = "send-sync", async_trait)]
    impl Transaction for RevokeProperty {
        type Error = OperationError;

        type Output = ();

        async fn build_programmable_transaction<C>(&self, client: &C) -> Result<ProgrammableTransaction, Self::Error>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            self.cached_ptb.get_or_try_init(|| self.make_ptb(client)).await.cloned()
        }

        async fn apply<C>(mut self, _: &mut IotaTransactionBlockEffects, _: &C) -> Result<Self::Output, Self::Error>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            Ok(())
        }
    }
}
