// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Create Federation Transaction
//!
//! This module provides the transaction implementation for creating new federations
//! in the ITH system. A federation serves as the root trust authority for a
//! hierarchical trust network.
//!
//! ## Overview
//!
//! The `CreateFederation` transaction creates a new shared federation object on the
//! IOTA network and grants the transaction sender all three initial capability types:
//! `RootAuthorityCap`, `AccreditCap`, and `AttestCap`. This establishes the sender
//! as the federation's root authority with full control over the trust hierarchy.

use async_trait::async_trait;
use iota_interaction::rpc_types::{IotaTransactionBlockEffects, IotaTransactionBlockEvents};
use iota_interaction::types::transaction::ProgrammableTransaction;
use iota_interaction::OptionalSync;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{ITHImpl, ITHOperations};
use crate::core::types::{Event, Federation, FederationCreatedEvent};
use crate::error::Error;

/// A transaction that creates a new federation.
#[derive(Debug, Clone)]
pub struct CreateFederation {
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl Default for CreateFederation {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateFederation {
    /// Creates a new [`CreateFederation`] instance.
    ///
    /// This creates a reusable transaction builder that can be used to create
    /// multiple federation creation transactions. The transaction is cached
    /// after the first build for efficiency.
    pub fn new() -> Self {
        Self {
            cached_ptb: OnceCell::new(),
        }
    }

    /// Builds the programmable transaction for creating a federation.
    ///
    /// This method creates the underlying Move transaction that will create
    /// the federation object and grant capabilities to the sender.
    ///
    /// # Parameters
    ///
    /// - `client`: The client providing the ITH package ID
    ///
    /// # Returns
    ///
    /// A `ProgrammableTransaction` ready for execution on the IOTA network.
    async fn make_ptb(&self, client: &impl CoreClientReadOnly) -> Result<ProgrammableTransaction, Error> {
        ITHImpl::new_federation(client.package_id())
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for CreateFederation {
    type Error = Error;

    type Output = Federation;

    async fn build_programmable_transaction<C>(&self, client: &C) -> Result<ProgrammableTransaction, Self::Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        self.cached_ptb.get_or_try_init(|| self.make_ptb(client)).await.cloned()
    }

    async fn apply_with_events<C>(
        mut self,
        _: &mut IotaTransactionBlockEffects,
        events: &mut IotaTransactionBlockEvents,
        client: &C,
    ) -> Result<Self::Output, Self::Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let events = events
            .data
            .first()
            .ok_or_else(|| Error::TransactionUnexpectedResponse("events should be provided".to_string()))?
            .parsed_json
            .clone();

        let event: Event<FederationCreatedEvent> = serde_json::from_value(events)
            .map_err(|e| Error::TransactionUnexpectedResponse(format!("failed to parse event: {e}")))?;

        let federation_id = event.data.federation_address;

        let federation = client
            .get_object_by_id(federation_id)
            .await
            .map_err(|e| Error::ObjectLookup(e.to_string()))?;

        Ok(federation)
    }

    async fn apply<C>(mut self, _: &mut IotaTransactionBlockEffects, _: &C) -> Result<Self::Output, Self::Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        unreachable!()
    }
}
