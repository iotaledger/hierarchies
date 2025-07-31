// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Client
//!
//! The full client provides read-write access to Hierarchies on the IOTA blockchain.
//!
//! ## Overview
//!
//! This client extends [`HierarchiesClientReadOnly`] with transaction capabilities,
//! allowing you to create, update, transfer, and destroy Hierarchies.
//!
//! ## Transaction Flow
//!
//! All transaction methods return a [`TransactionBuilder`] that follows this pattern:
//!
//! ```rust,ignore
//! # use hierarchies::client::full_client::HierarchiesClient;
//! # use hierarchies::core::types::State;
//! # use iota_interaction::types::base_types::ObjectID;
//! # async fn example(client: &HierarchiesClient<impl secret_storage::Signer<iota_interaction::IotaKeySignature>>) -> Result<(), Box<dyn std::error::Error>> {
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
//! # use hierarchies::core::builder::HierarchiesBuilder;
//! # use hierarchies::core::types::{State, TimeLock};
//! # use hierarchies::client::full_client::HierarchiesClient;
//! # async fn example(client: &HierarchiesClient<impl secret_storage::Signer<iota_interaction::IotaKeySignature>>) -> Result<(), Box<dyn std::error::Error>> {
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
use product_common::core_client::{CoreClient, CoreClientReadOnly};
use product_common::network_name::NetworkName;
use product_common::transaction::transaction_builder::TransactionBuilder;
use secret_storage::Signer;

use super::HierarchiesClientReadOnly;
use crate::client::error::ClientError;
use crate::core::transactions::add_root_authority::AddRootAuthority;
use crate::core::transactions::statements::add_statement::AddStatement;
use crate::core::transactions::statements::revoke_statement::RevokeStatement;
use crate::core::transactions::{
    CreateAccreditation, CreateAccreditationToAttest, CreateFederation, RevokeAccreditationToAccredit,
    RevokeAccreditationToAttest,
};
use crate::core::types::statements::name::StatementName;
use crate::core::types::statements::value::StatementValue;
use crate::core::types::statements::Statement;
use crate::iota_interaction_adapter::IotaClientAdapter;

/// The `HierarchiesClient` struct is responsible for managing the connection to the
/// IOTA network and executing transactions on behalf of the Hierarchies package.
pub struct HierarchiesClient<S> {
    read_client: HierarchiesClientReadOnly,
    /// The public key of the client.
    public_key: PublicKey,
    /// The signer of the client.
    signer: S,
}

impl<S> HierarchiesClient<S>
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
    /// # use hierarchies::client::full_client::{HierarchiesClient, HierarchiesClientReadOnly};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let read_client = HierarchiesClientReadOnly::new(adapter, package_id)?;
    /// let signer = get_signer()?; // Your signer implementation
    /// let client = HierarchiesClient::new(read_client, signer).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(client: HierarchiesClientReadOnly, signer: S) -> Result<Self, ClientError> {
        let public_key = signer.public_key().await.map_err(|e| ClientError::InvalidInput {
            details: format!("Invalid key: {e}"),
        })?;

        Ok(Self {
            public_key,
            read_client: client,
            signer,
        })
    }
}

impl<S> HierarchiesClient<S>
where
    S: Signer<IotaKeySignature> + OptionalSync,
{
    /// Creates a builder for a locked notarization.
    pub fn create_new_federation(&self) -> TransactionBuilder<CreateFederation> {
        TransactionBuilder::new(CreateFederation::new())
    }

    /// Creates a [`TransactionBuilder`] for adding a root authority to a federation.
    pub fn add_root_authority(
        &self,
        federation_id: ObjectID,
        account_id: ObjectID,
    ) -> TransactionBuilder<AddRootAuthority> {
        TransactionBuilder::new(AddRootAuthority::new(federation_id, account_id, self.sender_address()))
    }

    /// Creates a new [`AddStatement`] transaction builder.
    pub fn add_statement(
        &self,
        federation_id: ObjectID,
        statement_name: StatementName,
        allowed_statement_values: HashSet<StatementValue>,
        allow_any: bool,
    ) -> TransactionBuilder<AddStatement> {
        TransactionBuilder::new(AddStatement::new(
            federation_id,
            statement_name,
            allowed_statement_values,
            allow_any,
            self.sender_address(),
        ))
    }

    /// Creates a new [`RevokeStatement`] transaction builder.
    pub fn revoke_statement(
        &self,
        federation_id: ObjectID,
        statement_name: StatementName,
        valid_to_ms: Option<u64>,
    ) -> TransactionBuilder<RevokeStatement> {
        TransactionBuilder::new(RevokeStatement::new(
            federation_id,
            statement_name,
            valid_to_ms,
            self.sender_address(),
        ))
    }

    /// Creates a new [`CreateAccreditationToAttest`] transaction builder.
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

    /// Creates a new [`RevokeAccreditationToAttest`] transaction builder.
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

    /// Creates a new [`CreateAccreditation`] transaction builder.
    pub fn create_accreditation_to_accredit(
        &self,
        federation_id: ObjectID,
        receiver: ObjectID,
        statements: impl IntoIterator<Item = Statement>,
    ) -> TransactionBuilder<CreateAccreditation> {
        TransactionBuilder::new(CreateAccreditation::new(
            federation_id,
            receiver,
            statements.into_iter().collect(),
            self.sender_address(),
        ))
    }

    /// Creates a new [`RevokeAccreditationToAccredit`] transaction builder.
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

impl<S> Deref for HierarchiesClient<S> {
    type Target = HierarchiesClientReadOnly;

    fn deref(&self) -> &Self::Target {
        &self.read_client
    }
}

impl<S> CoreClientReadOnly for HierarchiesClient<S>
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

impl<S> CoreClient<S> for HierarchiesClient<S>
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
