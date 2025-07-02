// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Create Accreditation to Attest
//!
//! This module defines the create accreditation to attest transaction and operations.
//!
//! ## Overview
//!
//! This transaction grants attestation permissions to another user, allowing them
//! to create trusted attestations for the specified statements.

use async_trait::async_trait;
use iota_interaction::rpc_types::IotaTransactionBlockEffects;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::ProgrammableTransaction;
use iota_interaction::OptionalSync;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{ITHImpl, ITHOperations};
use crate::core::types::statements::Statement;
use crate::error::Error;

/// Transaction for creating accreditation to attest permissions.
///
/// This transaction allows a user with sufficient permissions to grant another user
/// the ability to create attestations for specific statements.
pub struct CreateAccreditationToAttest {
    /// The ID of the federation where the accreditation will be granted
    federation_id: ObjectID,
    /// The ID of the user who will receive the attestation permissions
    receiver: ObjectID,
    /// The statements for which attestation permissions are being granted
    want_statements: Vec<Statement>,
    /// The address of the signer (used for capability verification)
    signer_address: IotaAddress,
    /// Cached programmable transaction
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl CreateAccreditationToAttest {
    /// Creates a new [`CreateAccreditationToAttest`] instance.
    ///
    /// ## Arguments
    ///
    /// * `federation_id` - The ID of the federation where the accreditation will be granted
    /// * `receiver` - The ID of the user who will receive the attestation permissions
    /// * `want_statements` - The statements for which permissions are being granted
    /// * `signer_address` - The address of the signer (must have AttestCap)
    ///
    /// ## Returns
    ///
    /// A new instance of [`CreateAccreditationToAttest`]
    pub fn new(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_statements: impl IntoIterator<Item = Statement>,
        signer_address: IotaAddress,
    ) -> Self {
        Self {
            federation_id,
            receiver,
            want_statements: want_statements.into_iter().collect(),
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Makes a [`ProgrammableTransaction`] for the [`CreateAccreditationToAttest`] instance.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, Error>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb = ITHImpl::create_accreditation_to_attest(
            self.federation_id,
            self.receiver,
            self.want_statements.clone(),
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
