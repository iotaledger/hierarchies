// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A read-only client for interacting with IOTA Hierarchies module objects.
//!
//! This client provides methods to query the state and metadata of Hierarchies objects
//! on the IOTA network without requiring signing capabilities.

use std::ops::Deref;

#[cfg(not(target_arch = "wasm32"))]
use iota_interaction::IotaClient;
use iota_interaction::IotaClientTrait;
use iota_interaction::types::base_types::{IotaAddress, ObjectID};
use iota_interaction::types::transaction::{ProgrammableTransaction, TransactionKind};
#[cfg(target_arch = "wasm32")]
use iota_interaction_ts::bindings::WasmIotaClient;
use product_common::core_client::CoreClientReadOnly;
use product_common::network_name::NetworkName;
use product_common::package_registry::Env;
use serde::de::DeserializeOwned;

use crate::client::error::ClientError;
use crate::client::{get_object_ref_by_id_with_bcs, network_id};
use crate::core::operations::{HierarchiesImpl, HierarchiesOperations};
use crate::core::types::property_name::PropertyName;
use crate::core::types::property_value::PropertyValue;
use crate::core::types::{Accreditations, Federation};
use crate::error::ConfigError;
use crate::iota_interaction_adapter::IotaClientAdapter;
use crate::package;

/// A read-only client for the Hierarchies.
///
/// This client is used for communicating with the Hierarchies in a read-only manner.
///
/// This client supports both OffChain and OnChain operations on the Hierarchies.
#[derive(Clone)]
pub struct HierarchiesClientReadOnly {
    /// The underlying IOTA client adapter used for communication.
    client: IotaClientAdapter,
    /// The [`ObjectID`] of the deployed Hierarchies package (smart contract).
    /// All interactions go through this package ID.
    hierarchies_package_id: ObjectID,
    /// The name of the network this client is connected to (e.g., "mainnet", "testnet").
    network_name: NetworkName,
    chain_id: String,
}

impl Deref for HierarchiesClientReadOnly {
    type Target = IotaClientAdapter;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl HierarchiesClientReadOnly {
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

    /// Attempts to create a new [`HierarchiesClientReadOnly`] from a given IOTA client.
    ///
    /// # Failures
    /// This function fails if the provided `iota_client` is connected to an unrecognized
    /// network for which the Hierarchies package ID is not known in the internal
    /// package registry.
    ///
    /// # Arguments
    ///
    /// * `iota_client`: The IOTA client instance to use for communication. This can be either a native `IotaClient` or
    ///   a WASM-specific `WasmIotaClient`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the initialized [`HierarchiesClientReadOnly`] on success,
    /// or an [`ClientError`] if the network is unrecognized or communication fails.
    pub async fn new(
        #[cfg(target_arch = "wasm32")] iota_client: WasmIotaClient,
        #[cfg(not(target_arch = "wasm32"))] iota_client: IotaClient,
    ) -> Result<Self, ClientError> {
        let client = IotaClientAdapter::new(iota_client);
        let network = network_id(&client).await?;
        Self::new_internal(client, network).await
    }

    /// Internal helper function to create a new [`HierarchiesClientReadOnly`].
    ///
    /// This function looks up the Hierarchies package ID based on the provided network name
    /// using the internal package registry.
    async fn new_internal(iota_client: IotaClientAdapter, network: NetworkName) -> Result<Self, ClientError> {
        let chain_id = network.as_ref().to_string();
        let (network, hierarchies_pkg_id) = {
            let package_registry = package::hierarchies_package_registry().await;
            let package_id = package_registry.package_id(&network).ok_or_else(|| {
                ClientError::Configuration(ConfigError::PackageNotFound {
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
        Ok(HierarchiesClientReadOnly {
            client: iota_client,
            hierarchies_package_id: hierarchies_pkg_id,
            network_name: network,
            chain_id,
        })
    }

    /// Creates a new [`HierarchiesClientReadOnly`] with a specific Hierarchies package ID.
    ///
    /// This function allows overriding the package ID lookup from the registry,
    /// which is useful for connecting to networks where the package ID is known
    /// but not yet registered, or for testing with custom deployments.
    #[allow(deprecated)] // TODO : Remove after MoveHistoryManager is released with product-core
    pub async fn new_with_pkg_id(
        #[cfg(target_arch = "wasm32")] iota_client: WasmIotaClient,
        #[cfg(not(target_arch = "wasm32"))] iota_client: IotaClient,
        package_id: ObjectID,
    ) -> Result<Self, ClientError> {
        let client = IotaClientAdapter::new(iota_client);
        let network = network_id(&client).await?;

        // Use the passed pkg_id to add a new env or override the information of an existing one.
        {
            let mut registry = package::hierarchies_package_registry_mut().await;
            registry.insert_env_history(Env::new(network.as_ref()), vec![package_id]);
        }

        Self::new_internal(client, network).await
    }

    /// Retrieves a federation by its ID.
    pub async fn get_federation_by_id(&self, federation_id: ObjectID) -> Result<Federation, ClientError> {
        let fed = get_object_ref_by_id_with_bcs(self, &federation_id).await?;

        Ok(fed)
    }

    /// Check if root authority is in the federation.
    pub async fn is_root_authority(&self, federation_id: ObjectID, user_id: ObjectID) -> Result<bool, ClientError> {
        let tx = HierarchiesImpl::is_root_authority(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Retrieves all property names registered in the federation.
    pub async fn get_properties(&self, federation_id: ObjectID) -> Result<Vec<PropertyName>, ClientError> {
        let tx = HierarchiesImpl::get_properties(federation_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Checks if a property is registered in the federation.
    pub async fn is_property_in_federation(
        &self,
        federation_id: ObjectID,
        property_name: PropertyName,
    ) -> Result<bool, ClientError> {
        let tx = HierarchiesImpl::is_property_in_federation(federation_id, property_name, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Retrieves attestation accreditations for a specific user.
    pub async fn get_accreditations_to_attest(
        &self,
        federation_id: ObjectID,
        user_id: ObjectID,
    ) -> Result<Accreditations, ClientError> {
        let tx = HierarchiesImpl::get_accreditations_to_attest(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Checks if a user has attestation permissions.
    pub async fn is_attester(&self, federation_id: ObjectID, user_id: ObjectID) -> Result<bool, ClientError> {
        let tx = HierarchiesImpl::is_attester(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Retrieves accreditations to accredit for a specific user.
    pub async fn get_accreditations_to_accredit(
        &self,
        federation_id: ObjectID,
        user_id: ObjectID,
    ) -> Result<Accreditations, ClientError> {
        let tx = HierarchiesImpl::get_accreditations_to_accredit(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Checks if a user has accreditations to accredit.
    pub async fn is_accreditor(&self, federation_id: ObjectID, user_id: ObjectID) -> Result<bool, ClientError> {
        let tx = HierarchiesImpl::is_accreditor(federation_id, user_id, self).await?;
        let result = self.execute_read_only_transaction(tx).await?;
        Ok(result)
    }

    /// Validates an attestation
    pub async fn validate_property(
        &self,
        federation_id: ObjectID,
        attester_id: ObjectID,
        property_name: PropertyName,
        property_value: PropertyValue,
    ) -> Result<bool, ClientError> {
        let tx =
            HierarchiesImpl::validate_property(federation_id, attester_id, property_name, property_value, self).await?;

        let response = self.execute_read_only_transaction(tx).await?;
        Ok(response)
    }

    /// Validates an attestations
    pub async fn validate_properties(
        &self,
        federation_id: ObjectID,
        entity_id: ObjectID,
        properties: impl IntoIterator<Item = (PropertyName, PropertyValue)>,
    ) -> Result<bool, ClientError> {
        let tx = HierarchiesImpl::validate_properties(federation_id, entity_id, properties.into_iter().collect(), self)
            .await?;

        let response = self.execute_read_only_transaction(tx).await?;
        Ok(response)
    }
}

impl HierarchiesClientReadOnly {
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
    /// A `Result` containing the deserialized result of type `T` or an
    /// [`ClientError`].
    async fn execute_read_only_transaction<T: DeserializeOwned>(
        &self,
        tx: ProgrammableTransaction,
    ) -> Result<T, ClientError> {
        let inspection_result = self
            .client
            .read_api()
            .dev_inspect_transaction_block(IotaAddress::ZERO, TransactionKind::programmable(tx), None, None, None)
            .await
            .map_err(|err| ClientError::ExecutionFailed {
                reason: format!("Failed to inspect transaction block: {err}"),
            })?;

        let execution_results = inspection_result.results.ok_or_else(|| ClientError::InvalidResponse {
            reason: "DevInspectResults missing 'results' field".to_string(),
        })?;

        if execution_results.is_empty() {
            return Err(ClientError::InvalidResponse {
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

        Err(ClientError::InvalidResponse {
            reason: format!(
                "No execution result could be deserialized as expected type. Total results: {}. Last error: {}",
                execution_results.len(),
                last_error.map_or("No return values found".to_string(), |e| e.to_string())
            ),
        })
    }
}

#[async_trait::async_trait]
impl CoreClientReadOnly for HierarchiesClientReadOnly {
    fn package_id(&self) -> ObjectID {
        self.hierarchies_package_id
    }

    fn network_name(&self) -> &NetworkName {
        &self.network_name
    }

    fn client_adapter(&self) -> &IotaClientAdapter {
        &self.client
    }
}
