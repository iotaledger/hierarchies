// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Create Federation Transaction
//!
//! This module provides the transaction implementation for creating new federations
//! in the Hierarchies system. A federation serves as the root trust authority for a
//! hierarchical trust network.
//!
//! ## Overview
//!
//! The `CreateFederation` transaction creates a new shared federation object on the
//! IOTA network and grants the transaction sender two initial capability types:
//! `RootAuthorityCap` and `AccreditCap`. This establishes the sender
//! as the federation's root authority with full control over the trust hierarchy.

use async_trait::async_trait;
use iota_interaction::OptionalSync;
use iota_interaction::rpc_types::{IotaTransactionBlockEffects, IotaTransactionBlockEvents};
use iota_interaction::types::transaction::ProgrammableTransaction;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{HierarchiesImpl, HierarchiesOperations};
use crate::core::transactions::TransactionError;
use crate::core::types::Federation;
use crate::core::types::events::FederationCreatedEvent;

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
    /// # Returns
    /// A `ProgrammableTransaction` ready for execution on the IOTA network.
    async fn make_ptb(&self, client: &impl CoreClientReadOnly) -> Result<ProgrammableTransaction, TransactionError> {
        HierarchiesImpl::new_federation(client.package_id()).map_err(TransactionError::from)
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for CreateFederation {
    type Error = TransactionError;

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
            .ok_or_else(|| TransactionError::InvalidResponse)?
            .parsed_json
            .clone();

        let event: FederationCreatedEvent =
            serde_json::from_value(events).map_err(|_e| TransactionError::EventProcessingFailed {
                event_type: "FederationCreatedEvent".to_string(),
            })?;

        let federation_address = event.federation_address;

        let federation =
            client
                .get_object_by_id(federation_address)
                .await
                .map_err(|e| TransactionError::ExecutionFailed {
                    reason: format!("Failed to retrieve federation object: {e}"),
                })?;

        Ok(federation)
    }

    async fn apply<C>(mut self, _: &mut IotaTransactionBlockEffects, _: &C) -> Result<Self::Output, Self::Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        unreachable!()
    }
}
