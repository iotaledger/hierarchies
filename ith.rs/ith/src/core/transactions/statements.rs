//! # Statement Management Transactions
//!
//! This module provides transaction implementations for managing statements
//! within ITH federations. Statements define the types of claims that can
//! be attested within a federation.

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
use crate::core::types::statements::name::StatementName;
use crate::core::types::statements::value::StatementValue;
use crate::error::Error;

/// Transaction for adding new statement types to federations.
pub mod add_statement {
    use super::*;

    /// A transaction that adds a new statement type to a federation.
    ///
    /// This transaction allows root authorities to define new types of claims
    /// that can be attested within their federation. You can either restrict
    /// the allowed values to a specific set or allow any values.
    ///
    /// ## Requirements
    ///
    /// - The owner must possess `RootAuthorityCap` for the federation
    /// - The statement name must be unique within the federation
    #[derive(Debug, Clone)]
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
        /// # Parameters
        ///
        /// - `federation_id`: The ID of the federation where the statement will be added
        /// - `statement_name`: The unique name identifier for the statement type
        /// - `allowed_values`: Set of specific values permitted for this statement (ignored if `allow_any` is true)
        /// - `allow_any`: Whether to allow any values for this statement type
        /// - `owner`: Address that owns the required `RootAuthorityCap`
        ///
        /// # Returns
        ///
        /// A new `AddStatement` transaction instance ready for execution.
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

        /// Builds the programmable transaction for adding a statement.
        ///
        /// This method creates the underlying Move transaction that will add
        /// the new statement type to the federation with the specified constraints.
        ///
        /// # Parameters
        ///
        /// - `client`: The client providing access to the IOTA network
        ///
        /// # Returns
        ///
        /// A `ProgrammableTransaction` ready for execution on the IOTA network.
        ///
        /// # Errors
        ///
        /// Returns an error if the owner doesn't have `RootAuthorityCap` or if
        /// the statement name already exists in the federation.
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

/// Transaction for revoking statement types from federations.
pub mod revoke_statement {
    use super::*;

    /// A transaction that revokes a statement type from a federation.
    ///
    /// This transaction allows root authorities to revoke statement types,
    /// preventing future attestations of that type. You can either revoke
    /// immediately or schedule the revocation for a specific future time.
    ///
    /// ## Requirements
    ///
    /// - The owner must possess `RootAuthorityCap` for the federation
    /// - The statement must exist in the federation
    #[derive(Debug, Clone)]
    pub struct RevokeStatement {
        federation_id: ObjectID,
        statement_name: StatementName,
        valid_to_ms: Option<u64>,
        owner: IotaAddress,
        cached_ptb: OnceCell<ProgrammableTransaction>,
    }

    impl RevokeStatement {
        /// Creates a new [`RevokeStatement`] instance.
        ///
        /// # Parameters
        ///
        /// - `federation_id`: The ID of the federation containing the statement
        /// - `statement_name`: The name of the statement type to revoke
        /// - `valid_to_ms`: Optional timestamp in milliseconds when the statement should expire. If `None`, the
        ///   statement is revoked immediately using the current time.
        /// - `owner`: Address that owns the required `RootAuthorityCap`
        ///
        /// # Returns
        ///
        /// A new `RevokeStatement` transaction instance ready for execution.
        pub fn new(
            federation_id: ObjectID,
            statement_name: StatementName,
            valid_to_ms: Option<u64>,
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

        /// Builds the programmable transaction for revoking a statement.
        ///
        /// This method creates the underlying Move transaction that will revoke
        /// the statement type, either immediately or at a scheduled time.
        ///
        /// # Parameters
        ///
        /// - `client`: The client providing access to the IOTA network
        ///
        /// # Returns
        ///
        /// A `ProgrammableTransaction` ready for execution on the IOTA network.
        ///
        /// # Errors
        ///
        /// Returns an error if the owner doesn't have `RootAuthorityCap` or if
        /// the statement doesn't exist in the federation.
        async fn make_ptb<C>(&self, client: &C) -> Result<ProgrammableTransaction, Error>
        where
            C: CoreClientReadOnly + OptionalSync,
        {
            let ptb = match self.valid_to_ms {
                Some(valid_to_ms) => {
                    ITHImpl::revoke_statement_at(
                        self.federation_id,
                        self.statement_name.clone(),
                        valid_to_ms,
                        self.owner,
                        client,
                    )
                    .await?
                }
                None => {
                    ITHImpl::revoke_statement(self.federation_id, self.statement_name.clone(), self.owner, client)
                        .await?
                }
            };

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
