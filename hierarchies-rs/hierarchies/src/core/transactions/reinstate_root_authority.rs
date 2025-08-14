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
use iota_interaction::rpc_types::IotaTransactionBlockEffects;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::ProgrammableTransaction;
use iota_interaction::OptionalSync;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{HierarchiesImpl, HierarchiesOperations};
use crate::error::TransactionError;

/// A transaction that reinstates a previously revoked root authority to the federation.
///
/// This transaction allows an existing root authority to restore a revoked root authority
/// back to active status, granting them `RootAuthorityCap` again. The target account
/// must be in the federation's revoked list and cannot already be an active root authority.
///
/// ## Requirements
/// - The signer must already possess a `RootAuthorityCap` for the federation
/// - The target account must be in the revoked root authorities list
/// - The target account must not already be an active root authority
pub struct ReinstateRootAuthority {
    federation_id: ObjectID,
    account_id: ObjectID,
    signer_address: IotaAddress,
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl ReinstateRootAuthority {
    /// Creates a new [`ReinstateRootAuthority`] instance.
    ///
    /// # Returns
    ///
    /// A new `ReinstateRootAuthority` transaction instance ready for execution.
    pub fn new(federation_id: ObjectID, account_id: ObjectID, signer_address: IotaAddress) -> Self {
        Self {
            federation_id,
            account_id,
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Builds the programmable transaction for reinstating a root authority.
    ///
    /// This method creates the underlying Move transaction that will restore
    /// `RootAuthorityCap` to the target account by removing them from the revoked
    /// list and adding them back to the active root authorities.
    ///
    /// # Returns
    ///
    /// A `ProgrammableTransaction` ready for execution on the IOTA network.
    ///
    /// # Errors
    ///
    /// Returns an error if the signer doesn't have the required `RootAuthorityCap`
    /// or if the target account is not in the revoked list.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, TransactionError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb =
            HierarchiesImpl::reinstate_root_authority(self.federation_id, self.account_id, self.signer_address, client)
                .await?;

        Ok(ptb)
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for ReinstateRootAuthority {
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
