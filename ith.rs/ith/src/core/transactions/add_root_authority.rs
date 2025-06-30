// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Create Federation
//!
//! This module defines the create federation struct and the operations for create federation.
//!
//! ## Overview
//!
//! The create federation is a struct that contains the state, metadata, and operations for a create federation.

use async_trait::async_trait;
use iota_interaction::rpc_types::IotaTransactionBlockEffects;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::ProgrammableTransaction;
use iota_interaction::OptionalSync;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{ITHImpl, ITHOperations};
use crate::error::Error;

pub struct AddRootAuthority {
    federation_id: ObjectID,
    account_id: ObjectID,
    signer_address: IotaAddress,
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl AddRootAuthority {
    /// Creates a new [`AddRootAuthority`] instance.
    ///
    /// ## Arguments
    ///
    /// * `federation_id` - The ID of the federation where the root authority will be added.
    /// * `account_id` - The account ID of the root authority.
    /// * `signer_address` - The address of the signer (Used for the cap).
    ///
    /// ## Returns
    pub fn new(federation_id: ObjectID, account_id: ObjectID, signer_address: IotaAddress) -> Self {
        Self {
            federation_id,
            account_id,
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Makes a [`ProgrammableTransaction`] for the [`CreateFederation`] instance.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb = ITHImpl::add_root_authority(self.federation_id, self.account_id, self.signer_address, client).await?;

        Ok(ptb)
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for AddRootAuthority {
    type Error = Error;

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
