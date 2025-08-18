// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Create Accreditation to Accredit
//!
//! This module defines the create accreditation to accredit transaction and operations.
//!
//! ## Overview
//!
//! This transaction grants accreditation permissions to another user, allowing them
//! to further delegate accreditation rights for the specified properties.

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

/// Transaction for creating accreditation to accredit.
///
/// This transaction allows a user with sufficient permissions to grant another user
/// the ability to delegate accreditation rights for specific properties.
pub struct CreateAccreditation {
    /// The ID of the federation where the accreditation will be granted
    federation_id: ObjectID,
    /// The ID of the user who will receive the accreditation permissions
    receiver: ObjectID,
    /// The properties for which accreditation permissions are being granted
    want_properties: Vec<FederationProperty>,
    /// The address of the signer (used for capability verification)
    signer_address: IotaAddress,
    /// Cached programmable transaction
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl CreateAccreditation {
    /// Creates a new [`CreateAccreditation`] instance.
    pub fn new(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_properties: Vec<FederationProperty>,
        signer_address: IotaAddress,
    ) -> Self {
        Self {
            federation_id,
            receiver,
            want_properties,
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Makes a [`ProgrammableTransaction`] for the [`CreateAccreditation`] instance.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb = HierarchiesImpl::create_accreditation_to_accredit(
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
impl Transaction for CreateAccreditation {
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
