// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Create Accreditation to Attest
//!
//! This module defines the create accreditation to attest transaction and operations.
//!
//! ## Overview
//!
//! This transaction grants attestation permissions to another user, allowing them
//! to create trusted attestations for the specified properties.

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
use crate::core::types::property::FederationProperty;

/// Transaction for creating accreditation to attest.
///
/// This transaction allows a user with sufficient permissions to grant another user
/// the ability to create attestations for specific properties.
pub struct CreateAccreditationToAttest {
    /// The ID of the federation where the accreditation will be granted
    federation_id: ObjectID,
    /// The ID of the user who will receive the attestation
    receiver: ObjectID,
    /// The properties for which attestation is being granted
    want_properties: Vec<FederationProperty>,
    /// The address of the signer (used for capability verification)
    signer_address: IotaAddress,
    /// Cached programmable transaction
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl CreateAccreditationToAttest {
    /// Creates a new [`CreateAccreditationToAttest`] instance.
    pub fn new(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_properties: impl IntoIterator<Item = FederationProperty>,
        signer_address: IotaAddress,
    ) -> Self {
        Self {
            federation_id,
            receiver,
            want_properties: want_properties.into_iter().collect(),
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Makes a [`ProgrammableTransaction`] for the [`CreateAccreditationToAttest`] instance.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb = HierarchiesImpl::create_accreditation_to_attest(
            self.federation_id,
            self.receiver,
            self.want_properties.clone(),
            self.signer_address,
            client,
        )
        .await?;
        Ok(ptb)
    }
}

#[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
#[cfg_attr(feature = "send-sync", async_trait)]
impl Transaction for CreateAccreditationToAttest {
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
