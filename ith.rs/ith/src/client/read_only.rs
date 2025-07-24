// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A read-only client for interacting with IOTA ITH module objects.
//!
//! This client provides methods to query the state and metadata of ITH objects
//! on the IOTA network without requiring signing capabilities.

use std::ops::Deref;

use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::{ProgrammableTransaction, TransactionKind};
use iota_interaction::{IotaClient, IotaClientTrait};
use product_common::core_client::CoreClientReadOnly;
use product_common::network_name::NetworkName;
use product_common::package_registry::{Env, Metadata};
use serde::de::DeserializeOwned;

use crate::client::errors::ReadOnlyClientError;
use crate::client::{get_object_ref_by_id_with_bcs, network_id};
use crate::core::operations::{ITHImpl, ITHOperations};
use crate::core::types::statements::name::StatementName;
use crate::core::types::statements::value::StatementValue;
use crate::core::types::{Accreditations, Federation};
use crate::error::ConfigError;
use crate::iota_interaction_adapter::IotaClientAdapter;
use crate::package;

/// A read-only client for the ITH.
///
/// This client is used for communicating with the ITH in a read-only manner.
///
/// This client supports both OffChain and OnChain operations on the ITH.
#[derive(Clone)]
pub struct ITHClientReadOnly {
    /// The underlying IOTA client adapter used for communication.
    client: IotaClientAdapter,
    /// The [`ObjectID`] of the deployed ITH package (smart contract).
    /// All interactions go through this package ID.
    ith_package_id: ObjectID,
    /// The name of the network this client is connected to (e.g., "mainnet", "testnet").
    network_name: NetworkName,
    chain_id: String,
}

impl Deref for ITHClientReadOnly {
    type Target = IotaClientAdapter;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl ITHClientReadOnly {
    /// Returns the name of the network the client is connected to.
    ///
    /// This name is derived from the network ID of the IOTA node the client is connected to.
    /// For the IOTA Mainnet, this will typically be "iota".
    pub const fn network(&self) -> &NetworkName {
        &self.network_name
    }

    /// Returns the chain identifier for the network this client is connected to.
    ///
    /// This is the raw chain ID string obtained from the IOTA node's network ID.
    /// For the IOTA Mainnet, this will typically be "iota".
    ///
    /// Note: This might be different from the `network()` name if an alias is used.
    pub fn chain_id(&self) -> &str {
        &self.chain_id
    }

    /// Attempts to create a new [`ITHClientReadOnly`] from a given IOTA client.
    ///
    /// # Failures
    /// This function fails if the provided `iota_client` is connected to an unrecognized
    /// network for which the ITH package ID is not known in the internal
    /// package registry.
    ///
    /// # Arguments
    ///
    /// * `iota_client`: The IOTA client instance to use for communication. This can be either a native `IotaClient` or
    ///   a WASM-specific `WasmIotaClient`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the initialized [`ITHClientReadOnly`] on success,
    /// or an [`Error`] if the network is unrecognized or communication fails.
    pub async fn new(
        #[cfg(target_arch = "wasm32")] iota_client: WasmIotaClient,
        #[cfg(not(target_arch = "wasm32"))] iota_client: IotaClient,
    ) -> Result<Self, ReadOnlyClientError> {
        let client = IotaClientAdapter::new(iota_client);
        let network = network_id(&client).await?;
        Self::new_internal(client, network).await
    }

    /// Internal helper function to create a new [`ITHClientReadOnly`].
    ///
    /// This function looks up the ITH package ID based on the provided network name
    /// using the internal package registry.
    ///
    /// # Arguments
    ///
    /// * `iota_client`: The IOTA client adapter.
    /// * `network`: The name of the network.
    async fn new_internal(iota_client: IotaClientAdapter, network: NetworkName) -> Result<Self, ReadOnlyClientError> {
        let chain_id = network.as_ref().to_string();
        let (network, ith_pkg_id) = {
            let package_registry = package::ith_package_registry().await;
            let package_id = package_registry.package_id(&network).ok_or_else(|| {
                ReadOnlyClientError::Configuration(ConfigError::PackageNotFound {
                    network: network.to_string(),
                })
            })?;
            let network = match chain_id.as_str() {
                product_common::package_registry::MAINNET_CHAIN_ID => {
                    NetworkName::try_from("iota").expect("valid network name")
                }
                _ => package_registry
                    .chain_alias(&chain_id)
                    .and_then(|alias| NetworkName::try_from(alias).ok())
                    .unwrap_or(network),
            };

            (network, package_id)
        };
        Ok(ITHClientReadOnly {
            client: iota_client,
            ith_package_id: ith_pkg_id,
            network_name: network,
            chain_id,
        })
    }

    /// Creates a new [`ITHClientReadOnly`] with a specific ITH package ID.
    ///
    /// This function allows overriding the package ID lookup from the registry, which is useful
    /// for connecting to networks where the package ID is known but not yet registered, or
    /// for testing with custom deployments.
    ///
    /// # Arguments
    ///
    /// * `iota_client`: The IOTA client instance.
    /// * `package_id`: The specific [`ObjectID`] of the ITH package to use.
    ///
    /// # Returns
    /// A `Result` containing the initialized [`ITHClientReadOnly`] or an [`Error`].
    pub async fn new_with_pkg_id(
        #[cfg(target_arch = "wasm32")] iota_client: WasmIotaClient,
        #[cfg(not(target_arch = "wasm32"))] iota_client: IotaClient,
        package_id: ObjectID,
    ) -> Result<Self, ReadOnlyClientError> {
        let client = IotaClientAdapter::new(iota_client);
        let network = network_id(&client).await?;

        // Use the passed pkg_id to add a new env or override the information of an existing one.
        {
            let mut registry = package::ith_package_registry_mut().await;
            registry.insert_env(Env::new(network.as_ref()), Metadata::from_package_id(package_id));
        }

        Self::new_internal(client, network).await
    }

    /// Retrieves a federation by its ID.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    ///
    /// # Returns
    /// A `Result` containing the [`Federation`] object or an [`Error`].
    pub async fn get_federation_by_id(&self, federation_id: ObjectID) -> Result<Federation, ReadOnlyClientError> {
        let fed = get_object_ref_by_id_with_bcs(self, &federation_id).await?;

        Ok(fed)
    }

    /// Retrieves all statement names registered in the federation.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    ///
    /// # Returns
    /// A `Result` containing the list of statement names or an [`Error`].
    pub async fn get_statements(&self, federation_id: ObjectID) -> Result<Vec<StatementName>, ReadOnlyClientError> {
        let tx = ITHImpl::get_statements(federation_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Checks if a statement is registered in the federation.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `statement_name`: The name of the statement to check.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the statement is registered or an [`Error`].
    pub async fn is_statement_in_federation(
        &self,
        federation_id: ObjectID,
        statement_name: StatementName,
    ) -> Result<bool, ReadOnlyClientError> {
        let tx = ITHImpl::is_statement_in_federation(federation_id, statement_name, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Retrieves attestation accreditations for a specific user.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    ///
    /// # Returns
    /// A `Result` containing the attestation accreditations or an [`Error`].
    pub async fn get_accreditations_to_attest(
        &self,
        federation_id: ObjectID,
        user_id: ObjectID,
    ) -> Result<Accreditations, ReadOnlyClientError> {
        let tx = ITHImpl::get_accreditations_to_attest(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Checks if a user has attestation permissions.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the user has attestation permissions or an [`Error`].
    pub async fn is_attester(&self, federation_id: ObjectID, user_id: ObjectID) -> Result<bool, ReadOnlyClientError> {
        let tx = ITHImpl::is_attester(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Retrieves accreditations to accredit for a specific user.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    ///
    /// # Returns
    /// A `Result` containing the accreditations to accredit or an [`Error`].
    pub async fn get_accreditations_to_accredit(
        &self,
        federation_id: ObjectID,
        user_id: ObjectID,
    ) -> Result<Accreditations, ReadOnlyClientError> {
        let tx = ITHImpl::get_accreditations_to_accredit(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Checks if a user has accreditations to accredit.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the user has accreditations to accredit or an [`Error`].
    pub async fn is_accreditor(&self, federation_id: ObjectID, user_id: ObjectID) -> Result<bool, ReadOnlyClientError> {
        let tx = ITHImpl::is_accreditor(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Validates a statement for a specific user.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `attester_id`: The [`ObjectID`] of the attester.
    /// * `statement_name`: The name of the statement to validate.
    /// * `statement_value`: The value of the statement to validate.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the statement is valid or an [`Error`].
    pub async fn validate_statement(
        &self,
        federation_id: ObjectID,
        attester_id: ObjectID,
        statement_name: StatementName,
        statement_value: StatementValue,
    ) -> Result<bool, ReadOnlyClientError> {
        let tx = ITHImpl::validate_statement(federation_id, attester_id, statement_name, statement_value, self).await?;

        let response = self.execute_read_only_transaction(tx).await?;
        Ok(response)
    }

    /// Validates multiple statements for a specific user.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    /// * `statements`: The statements to validate.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the statements are valid or an [`Error`].
    pub async fn validate_statements(
        &self,
        federation_id: ObjectID,
        entity_id: ObjectID,
        statements: impl IntoIterator<Item = (StatementName, StatementValue)>,
    ) -> Result<bool, ReadOnlyClientError> {
        let tx = ITHImpl::validate_statements(federation_id, entity_id, statements.into_iter().collect(), self).await?;

        let response = self.execute_read_only_transaction(tx).await?;
        Ok(response)
    }
}

impl ITHClientReadOnly {
    /// A helper function to execute a read-only transaction and deserialize
    /// the result into the specified type `T`.
    ///
    /// This function uses the `dev_inspect_transaction_block` endpoint of the IOTA client
    /// to simulate the execution of a programmable transaction without submitting it
    /// to the network.
    ///
    /// **Hybrid Strategy:**
    /// Tries execution results in reverse order (last to first), which prioritizes
    /// the most likely result while still checking all possibilities.
    ///
    /// # Arguments
    ///
    /// * `tx`: The [`ProgrammableTransaction`] to execute.
    ///
    /// # Returns
    /// A `Result` containing the deserialized result of type `T` or an [`Error`].
    async fn execute_read_only_transaction<T: DeserializeOwned>(
        &self,
        tx: ProgrammableTransaction,
    ) -> Result<T, ReadOnlyClientError> {
        let inspection_result = self
            .client
            .read_api()
            .dev_inspect_transaction_block(IotaAddress::ZERO, TransactionKind::programmable(tx), None, None, None)
            .await
            .map_err(|err| ReadOnlyClientError::ExecutionFailed {
                reason: format!("Failed to inspect transaction block: {err}"),
            })?;

        let execution_results = inspection_result
            .results
            .ok_or_else(|| ReadOnlyClientError::InvalidResponse {
                reason: "DevInspectResults missing 'results' field".to_string(),
            })?;

        if execution_results.is_empty() {
            return Err(ReadOnlyClientError::InvalidResponse {
                reason: "Execution results list is empty".to_string(),
            });
        }

        // Try execution results in reverse order (last to first)
        let mut last_error = None;

        for (_, result) in execution_results.iter().enumerate().rev() {
            // By default, the last return results will be from the function.
            if let Some((return_value_bytes, _)) = result.return_values.first() {
                match bcs::from_bytes::<T>(return_value_bytes) {
                    Ok(deserialized) => {
                        return Ok(deserialized);
                    }
                    Err(e) => {
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(ReadOnlyClientError::InvalidResponse {
            reason: format!(
                "No execution result could be deserialized as expected type. Total results: {}. Last error: {}",
                execution_results.len(),
                last_error.map_or("No return values found".to_string(), |e| e.to_string())
            ),
        })
    }
}

#[async_trait::async_trait]
impl CoreClientReadOnly for ITHClientReadOnly {
    fn package_id(&self) -> ObjectID {
        self.ith_package_id
    }

    fn network_name(&self) -> &NetworkName {
        &self.network_name
    }

    fn client_adapter(&self) -> &IotaClientAdapter {
        &self.client
    }
}
