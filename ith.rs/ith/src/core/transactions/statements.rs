// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use async_trait::async_trait;
use iota_interaction::rpc_types::IotaTransactionBlockEffects;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::ProgrammableTransaction;
use iota_interaction::OptionalSync;
use product_common::core_client::CoreClientReadOnly;
use product_common::transaction::transaction_builder::Transaction;
use tokio::sync::OnceCell;

use crate::core::operations::{ITHImpl, ITHOperations};
use crate::core::types::statements::{name::StatementName, value::StatementValue};
use crate::error::Error;

pub mod add_statement {
    use super::*;

    pub struct AddStatement {
        federation_id: ObjectID,
        statement_name: StatementName,
        allowed_values: HashSet<StatementValue>,
        allow_any: bool,
        owner: IotaAddress,
        cached_ptb: OnceCell<ProgrammableTransaction>,
    }

    impl AddStatement {
        /// Creates a new [`AddStatement`] instance.
        ///
        /// ## Arguments
        ///
        /// * `statement_name` - The name of the statement.
        /// * `allowed_values` - The allowed values of the statement.
        /// * `allow_any` - Whether to allow any value.
        ///
        /// ## Returns
        pub fn new(
            federation_id: ObjectID,
            statement_name: StatementName,
            allowed_values: HashSet<StatementValue>,
            allow_any: bool,
            owner: IotaAddress,
        ) -> Self {
            Self {
                federation_id,
                statement_name,
                allowed_values,
                allow_any,
                owner,
                cached_ptb: OnceCell::new(),
            }
        }

        /// Makes a [`ProgrammableTransaction`] for the [`AddStatement`] instance.
        async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, Error>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            let ptb = ITHImpl::add_statement(
                self.federation_id,
                self.statement_name.clone(),
                self.allowed_values.clone(),
                self.allow_any,
                self.owner,
                client,
            )
            .await?;

            Ok(ptb)
        }
    }

    #[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
    #[cfg_attr(feature = "send-sync", async_trait)]
    impl Transaction for AddStatement {
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
}

pub mod revoke_statement {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct RevokeStatement {
        federation_id: ObjectID,
        statement_name: StatementName,
        valid_to_ms: u64,
        owner: IotaAddress,
        cached_ptb: OnceCell<ProgrammableTransaction>,
    }

    impl RevokeStatement {
        pub fn new(
            federation_id: ObjectID,
            statement_name: StatementName,
            valid_to_ms: u64,
            owner: IotaAddress,
        ) -> Self {
            Self {
                federation_id,
                statement_name,
                valid_to_ms,
                owner,
                cached_ptb: OnceCell::new(),
            }
        }

        async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, Error>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            let ptb = ITHImpl::revoke_statement(
                self.federation_id,
                self.statement_name.clone(),
                self.valid_to_ms,
                self.owner,
                client,
            )
            .await?;
            Ok(ptb)
        }
    }

    #[cfg_attr(not(feature = "send-sync"), async_trait(?Send))]
    #[cfg_attr(feature = "send-sync", async_trait)]
    impl Transaction for RevokeStatement {
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
}
