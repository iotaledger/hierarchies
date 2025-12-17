// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Reinstate Root Authority Transaction
//!
//! This module provides the transaction implementation for reinstating previously
//! revoked root authorities in the Hierarchies system.
//!
//! ## Overview
//!
//! The `ReinstateRootAuthority` transaction allows existing root authorities to
//! restore a revoked root authority back to active status. This operation requires
//! the target account to be in the federation's revoked list and ensures it is
//! not already an active root authority.

use async_trait::async_trait;
use iota_interaction::OptionalSync;
use iota_interaction::rpc_types::IotaTransactionBlockEffects;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::ProgrammableTransaction;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{HierarchiesImpl, HierarchiesOperations};
use crate::core::types::property_name::PropertyName;
use crate::core::types::property_value::PropertyValue;
use crate::error::TransactionError;

/// A transaction that validates a property against the federation.
///
/// This transaction allows an attester to validate a property against the federation.
///
/// ## Requirements
/// - The attester must have a valid accreditation to attest to the property
/// - The property must exist in the federation
/// - The property must be valid according to the federation rules
pub struct ValidateProperty {
    federation_id: ObjectID,
    attester_id: ObjectID,
    property_name: PropertyName,
    property_value: PropertyValue,
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl ValidateProperty {
    /// Creates a new [`ValidateProperty`] instance.
    ///
    /// # Returns
    ///
    /// A new `ValidateProperty` transaction instance ready for execution.
    pub fn new(
        federation_id: ObjectID,
        attester_id: ObjectID,
        property_name: PropertyName,
        property_value: PropertyValue,
    ) -> Self {
        Self {
            federation_id,
            attester_id,
            property_name,
            property_value,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Builds the programmable transaction for validating a property.
    ///
    /// This method creates the underlying Move transaction that will validate
    /// the property against the federation.
    ///
    /// # Returns
    ///
    /// A `ProgrammableTransaction` ready for execution on the IOTA network.
    ///
    /// # Errors
    ///
    /// Returns an error if the attester doesn't have the required accreditation
    /// or if the property does not exist in the federation.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, TransactionError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb = HierarchiesImpl::validate_property(
            self.federation_id,
            self.attester_id,
            self.property_name.clone(),
            self.property_value.clone(),
            client,
        )
        .await?;
        Ok(ptb)
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for ValidateProperty {
    type Error = TransactionError;
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
