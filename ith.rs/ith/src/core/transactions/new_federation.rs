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
    pub fn new() -> Self {
        Self {
            cached_ptb: OnceCell::new(),
        }
    }

    /// Makes a [`ProgrammableTransaction`] for the [`CreateFederation`] instance.
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
