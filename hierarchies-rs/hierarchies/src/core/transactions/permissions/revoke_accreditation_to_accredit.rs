// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Revoke Accreditation to Accredit
//!
//! This module defines the revoke accreditation to accredit transaction and operations.
//!
//! ## Overview
//!
//! This transaction revokes accreditation permissions from a user, removing their
//! ability to delegate accreditation rights for specific statements.

use async_trait::async_trait;
use iota_interaction::rpc_types::IotaTransactionBlockEffects;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::ProgrammableTransaction;
use iota_interaction::OptionalSync;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{HierarchiesImpl, HierarchiesOperations};
use crate::core::OperationError;

/// Transaction for revoking accreditation to accredit permissions.
///
/// This transaction allows a user with sufficient permissions to revoke another user's
/// ability to delegate accreditation rights for specific statements.
pub struct RevokeAccreditationToAccredit {
    /// The ID of the federation where the accreditation will be revoked
    federation_id: ObjectID,
    /// The ID of the user whose accreditation permissions will be revoked
    user_id: ObjectID,
    /// The ID of the specific permission/accreditation to revoke
    permission_id: ObjectID,
    /// The address of the signer (used for capability verification)
    signer_address: IotaAddress,
    /// Cached programmable transaction
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl RevokeAccreditationToAccredit {
    /// Creates a new [`RevokeAccreditationToAccredit`] instance.
    pub fn new(
        federation_id: ObjectID,
        user_id: ObjectID,
        permission_id: ObjectID,
        signer_address: IotaAddress,
    ) -> Self {
        Self {
            federation_id,
            user_id,
            permission_id,
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Makes a [`ProgrammableTransaction`] for the [`RevokeAccreditationToAccredit`] instance.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb = HierarchiesImpl::revoke_accreditation_to_accredit(
            self.federation_id,
            self.user_id,
            self.permission_id,
            self.signer_address,
            client,
        )
        .await?;

        Ok(ptb)
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for RevokeAccreditationToAccredit {
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
