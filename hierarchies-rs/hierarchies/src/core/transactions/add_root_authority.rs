// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Add Root Authority Transaction
//!
//! This module provides the transaction implementation for adding new root authorities
//! to an existing federation in the Hierarchies system.
//!
//! ## Overview
//!
//! The `AddRootAuthority` transaction grants root authority capabilities to a new
//! account within a federation. Root authorities have the highest level of trust
//! and can perform all operations within the federation, including adding other
//! root authorities and managing statements.

use async_trait::async_trait;
use iota_interaction::OptionalSync;
use iota_interaction::rpc_types::IotaTransactionBlockEffects;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::ProgrammableTransaction;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{HierarchiesImpl, HierarchiesOperations};
use crate::error::TransactionError;

/// A transaction that adds a new root authority to an existing federation.
///
/// This transaction grants `RootAuthorityCap` to a new account, allowing them
/// to perform all federation operations including adding other root authorities,
/// managing properties, and creating accreditations.
///
/// ## Requirements
/// - The signer must already possess a `RootAuthorityCap` for the federation
/// - The target account must not already have root authority capabilities
pub struct AddRootAuthority {
    federation_id: ObjectID,
    account_id: ObjectID,
    signer_address: IotaAddress,
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl AddRootAuthority {
    /// Creates a new [`AddRootAuthority`] instance.
    ///
    /// # Returns
    ///
    /// A new `AddRootAuthority` transaction instance ready for execution.
    pub fn new(federation_id: ObjectID, account_id: ObjectID, signer_address: IotaAddress) -> Self {
        Self {
            federation_id,
            account_id,
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Builds the programmable transaction for adding a root authority.
    ///
    /// This method creates the underlying Move transaction that will grant
    /// `RootAuthorityCap` to the target account.
    ///
    /// # Returns
    ///
    /// A `ProgrammableTransaction` ready for execution on the IOTA network.
    ///
    /// # Errors
    ///
    /// Returns an error if the signer doesn't have the required `RootAuthorityCap`.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, TransactionError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb = HierarchiesImpl::add_root_authority(self.federation_id, self.account_id, self.signer_address, client)
            .await?;

        Ok(ptb)
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for AddRootAuthority {
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
