// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # ITH Client
//!
//! The full client provides read-write access to ITH on the IOTA blockchain.
//!
//! ## Overview
//!
//! This client extends [`ITHClientReadOnly`] with transaction capabilities,
//! allowing you to create, update, transfer, and destroy ITH.
//!
//! ## Transaction Flow
//!
//! All transaction methods return a [`TransactionBuilder`] that follows this pattern:
//!
//! ```rust,ignore
//! # use ith::client::full_client::ITHClient;
//! # use ith::core::types::State;
//! # use iota_interaction::types::base_types::ObjectID;
//! # async fn example(client: &ITHClient<impl secret_storage::Signer<iota_interaction::IotaKeySignature>>) -> Result<(), Box<dyn std::error::Error>> {
//! # let object_id = ObjectID::ZERO;
//! // 1. Create the transaction
//! let result = client
//!     .update_state(State::from_string("New data".to_string(), None), object_id)
//!     // 2. Configure transaction parameters (all optional)
//!     .with_gas_budget(1_000_000)     // Set custom gas budget
//!     .with_sender(sender_address)     // Override sender address
//!     .with_gas_payment(vec![coin])   // Use specific coins for gas
//!     // 3. Build and execute
//!     .build_and_execute(&client)      // Signs and submits transaction
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Available Configuration Methods
//!
//! The [`TransactionBuilder`] provides these configuration methods:
//! - `with_gas_budget(amount)` - Set gas budget (default: estimated)
//! - `with_gas_payment(coins)` - Use specific coins for gas payment
//! - `with_gas_owner(address)` - Set gas payer (default: sender)
//! - `with_gas_price(price)` - Override gas price (default: network price)
//! - `with_sender(address)` - Override transaction sender
//! - `with_sponsor(callback)` - Have another party pay for gas
//!
//! ## Example: Complete Notarization Workflow
//!
//! ```rust,ignore
//! # use ith::core::builder::ITHBuilder;
//! # use ith::core::types::{State, TimeLock};
//! # use ith::client::full_client::ITHClient;
//! # async fn example(client: &ITHClient<impl secret_storage::Signer<iota_interaction::IotaKeySignature>>) -> Result<(), Box<dyn std::error::Error>> {
//! // 1. Create a new federation
//! let create_result = client
//!     .create_new_federation()
//!     .build_and_execute(&client)
//!     .await?;
//!
//! let object_id = create_result.output;
//!
//! // 2. Add a root authority
//! client
//!     .add_root_authority(object_id, account_id)
//!     .build_and_execute(&client)
//!     .await?;
//!
//! # Ok(())
//! # }
//! ```

use std::collections::HashSet;
use std::ops::Deref;

use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::crypto::PublicKey;
use iota_interaction::{IotaKeySignature, OptionalSync};
use iota_interaction_rust::IotaClientAdapter;
use product_common::core_client::{CoreClient, CoreClientReadOnly};
use product_common::network_name::NetworkName;
use product_common::transaction::transaction_builder::TransactionBuilder;
use secret_storage::Signer;

use super::ITHClientReadOnly;
use crate::core::transactions::add_root_authority::AddRootAuthority;
use crate::core::transactions::statements::add_statement::AddStatement;
use crate::core::transactions::statements::remove_statement::RemoveStatement;
use crate::core::transactions::{
    CreateAccreditationToAccredit, CreateAccreditationToAttest, CreateFederation, RevokeAccreditationToAccredit,
    RevokeAccreditationToAttest,
};
use crate::core::types::{Statement, StatementName, StatementValue};
use crate::error::Error;

/// The `ITHClient` struct is responsible for managing the connection to the
/// IOTA network and executing transactions on behalf of the ITH package.
pub struct ITHClient<S> {
    read_client: ITHClientReadOnly,
    /// The public key of the client.
    public_key: PublicKey,
    /// The signer of the client.
    signer: S,
}

impl<S> ITHClient<S>
where
    S: Signer<IotaKeySignature>,
{
    /// Creates a new client with signing capabilities.
    ///
    /// ## Parameters
    ///
    /// - `client`: A read-only client for blockchain interaction
    /// - `signer`: A signer for transaction authorization
    ///
    /// ## Errors
    ///
    /// Returns an error if the signer's public key cannot be retrieved.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// # use ith::client::full_client::{ITHClient, ITHClientReadOnly};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let read_client = ITHClientReadOnly::new(adapter, package_id)?;
    /// let signer = get_signer()?; // Your signer implementation
    /// let client = ITHClient::new(read_client, signer).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(client: ITHClientReadOnly, signer: S) -> Result<Self, Error> {
        let public_key = signer
            .public_key()
            .await
            .map_err(|e| Error::InvalidKey(e.to_string()))?;

        Ok(Self {
            public_key,
            read_client: client,
            signer,
        })
    }
}

impl<S> ITHClient<S>
where
    S: Signer<IotaKeySignature> + OptionalSync,
{
    /// Creates a builder for a locked notarization.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// # use ith::client::full_client::ITHClient;
    /// # async fn example(client: &ITHClient<impl secret_storage::Signer<iota_interaction::IotaKeySignature>>) -> Result<(), Box<dyn std::error::Error>> {
    /// let result = client
    ///     .create_new_federation()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See [`ITHClient::create_new_federation`] for configuration options.
    pub fn create_new_federation(&self) -> TransactionBuilder<CreateFederation> {
        TransactionBuilder::new(CreateFederation::new())
    }

    /// Creates a [`TransactionBuilder`] for adding a root authority to a federation.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// # use ith::client::full_client::ITHClient;
    /// # async fn example(client: &ITHClient<impl secret_storage::Signer<iota_interaction::IotaKeySignature>>) -> Result<(), Box<dyn std::error::Error>> {
    /// let result = client
    ///     .add_root_authority(federation_id, account_id)
    ///     .build_and_execute(&client)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_root_authority(
        &self,
        federation_id: ObjectID,
        account_id: ObjectID,
    ) -> TransactionBuilder<AddRootAuthority> {
        TransactionBuilder::new(AddRootAuthority::new(federation_id, account_id, self.sender_address()))
    }

    pub fn add_statement(
        &self,
        federation_id: ObjectID,
        statement_name: StatementName,
        allowed_values: HashSet<StatementValue>,
        allow_any: bool,
    ) -> TransactionBuilder<AddStatement> {
        TransactionBuilder::new(AddStatement::new(
            federation_id,
            statement_name,
            allowed_values,
            allow_any,
            self.sender_address(),
        ))
    }

    pub fn remove_statement(
        &self,
        federation_id: ObjectID,
        statement_name: StatementName,
    ) -> TransactionBuilder<RemoveStatement> {
        TransactionBuilder::new(RemoveStatement::new(
            federation_id,
            statement_name,
            self.sender_address(),
        ))
    }

    pub fn create_accreditation_to_attest(
        &self,
        federation_id: ObjectID,
        receiver: ObjectID,
        want_statements: impl IntoIterator<Item = Statement>,
    ) -> TransactionBuilder<CreateAccreditationToAttest> {
        TransactionBuilder::new(CreateAccreditationToAttest::new(
            federation_id,
            receiver,
            want_statements,
            self.sender_address(),
        ))
    }

    pub fn revoke_accreditation_to_attest(
        &self,
        federation_id: ObjectID,
        user_id: ObjectID,
        permission_id: ObjectID,
    ) -> TransactionBuilder<RevokeAccreditationToAttest> {
        TransactionBuilder::new(RevokeAccreditationToAttest::new(
            federation_id,
            user_id,
            permission_id,
            self.sender_address(),
        ))
    }

    pub fn create_accreditation_to_accredit(
        &self,
        federation_id: ObjectID,
        receiver: ObjectID,
        want_statements: impl IntoIterator<Item = Statement>,
    ) -> TransactionBuilder<CreateAccreditationToAccredit> {
        TransactionBuilder::new(CreateAccreditationToAccredit::new(
            federation_id,
            receiver,
            want_statements.into_iter().collect(),
            self.sender_address(),
        ))
    }

    pub fn revoke_accreditation_to_accredit(
        &self,
        federation_id: ObjectID,
        user_id: ObjectID,
        permission_id: ObjectID,
    ) -> TransactionBuilder<RevokeAccreditationToAccredit> {
        TransactionBuilder::new(RevokeAccreditationToAccredit::new(
            federation_id,
            user_id,
            permission_id,
            self.sender_address(),
        ))
    }
}

impl<S> Deref for ITHClient<S> {
    type Target = ITHClientReadOnly;

    fn deref(&self) -> &Self::Target {
        &self.read_client
    }
}

impl<S> CoreClientReadOnly for ITHClient<S>
where
    S: OptionalSync,
{
    fn client_adapter(&self) -> &IotaClientAdapter {
        &self.read_client
    }

    fn package_id(&self) -> ObjectID {
        self.read_client.package_id()
    }

    fn network_name(&self) -> &NetworkName {
        self.read_client.network()
    }
}

impl<S> CoreClient<S> for ITHClient<S>
where
    S: Signer<IotaKeySignature> + OptionalSync,
{
    fn sender_address(&self) -> IotaAddress {
        IotaAddress::from(&self.public_key)
    }

    fn signer(&self) -> &S {
        &self.signer
    }

    fn sender_public_key(&self) -> &PublicKey {
        &self.public_key
    }
}
