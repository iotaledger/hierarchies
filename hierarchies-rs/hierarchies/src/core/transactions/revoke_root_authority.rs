// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Revoke Root Authority Transaction
//!
//! This module provides the transaction implementation for revoking root authorities
//! from an existing federation in the Hierarchies system.
//!
//! ## Overview
//!
//! The `RevokeRootAuthority` transaction removes root authority capabilities from an
//! account within a federation. The revoked authority's capability remains but becomes
//! unusable as they are added to a revocation list.

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

/// A transaction that revokes a root authority from an existing federation.
///
/// This transaction removes an account from the list of root authorities and adds
/// them to a revocation list, making their `RootAuthorityCap` unusable for future
/// operations.
///
/// ## Requirements
/// - The signer must already possess a `RootAuthorityCap` for the federation
/// - The target account must be an existing root authority
/// - Cannot revoke the last root authority (to prevent lockout)
pub struct RevokeRootAuthority {
    federation_id: ObjectID,
    account_id: ObjectID,
    signer_address: IotaAddress,
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl RevokeRootAuthority {
    /// Creates a new [`RevokeRootAuthority`] instance.
    ///
    /// # Returns
    ///
    /// A new `RevokeRootAuthority` transaction instance ready for execution.
    pub fn new(federation_id: ObjectID, account_id: ObjectID, signer_address: IotaAddress) -> Self {
        Self {
            federation_id,
            account_id,
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Builds the programmable transaction for revoking a root authority.
    ///
    /// This method creates the underlying Move transaction that will revoke
    /// root authority from the target account.
    ///
    /// # Returns
    ///
    /// A `ProgrammableTransaction` ready for execution on the IOTA network.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The signer doesn't have the required `RootAuthorityCap`
    /// - The target account is not a root authority
    /// - Attempting to revoke the last root authority
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, TransactionError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb =
            HierarchiesImpl::revoke_root_authority(self.federation_id, self.account_id, self.signer_address, client)
                .await?;

        Ok(ptb)
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for RevokeRootAuthority {
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
