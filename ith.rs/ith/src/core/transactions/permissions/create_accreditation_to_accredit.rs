// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Create Accreditation to Accredit
//!
//! This module defines the create accreditation to accredit transaction and operations.
//!
//! ## Overview
//!
//! This transaction grants accreditation permissions to another user, allowing them
//! to further delegate accreditation rights for the specified statements.

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
use crate::core::OperationError;

/// Transaction for creating accreditation to accredit permissions.
///
/// This transaction allows a user with sufficient permissions to grant another user
/// the ability to delegate accreditation rights for specific statements.
pub struct CreateAccreditation {
    /// The ID of the federation where the accreditation will be granted
    federation_id: ObjectID,
    /// The ID of the user who will receive the accreditation permissions
    receiver: ObjectID,
    /// The statements for which accreditation permissions are being granted
    want_statements: Vec<Statement>,
    /// The address of the signer (used for capability verification)
    signer_address: IotaAddress,
    /// Cached programmable transaction
    cached_ptb: OnceCell<ProgrammableTransaction>,
}

impl CreateAccreditation {
    /// Creates a new [`CreateAccreditationToAccredit`] instance.
    ///
    /// ## Arguments
    ///
    /// * `federation_id` - The ID of the federation where the accreditation will be granted
    /// * `receiver` - The ID of the user who will receive the accreditation permissions
    /// * `want_statements` - The statements for which permissions are being granted
    /// * `signer_address` - The address of the signer (must have AccreditCap)
    ///
    /// ## Returns
    ///
    /// A new instance of [`CreateAccreditationToAccredit`]
    pub fn new(
        federation_id: ObjectID,
        receiver: ObjectID,
        want_statements: Vec<Statement>,
        signer_address: IotaAddress,
    ) -> Self {
        Self {
            federation_id,
            receiver,
            want_statements,
            signer_address,
            cached_ptb: OnceCell::new(),
        }
    }

    /// Makes a [`ProgrammableTransaction`] for the [`CreateAccreditationToAccredit`] instance.
    async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, OperationError>
    where
        C: CoreClientReadOnly + OptionalSync,
    {
        let ptb = ITHImpl::create_accreditation_to_accredit(
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
