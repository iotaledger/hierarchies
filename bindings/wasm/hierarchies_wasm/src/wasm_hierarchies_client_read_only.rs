// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-1.0

use std::str::FromStr;

use anyhow::anyhow;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction_ts::bindings::WasmIotaClient;
use iota_interaction_ts::wasm_error::{wasm_error, Result, WasmResult};

use ith::client::ITHClientReadOnly;
use product_common::bindings::utils::parse_wasm_object_id;
use product_common::bindings::WasmObjectID;
use product_common::core_client::CoreClientReadOnly;
use wasm_bindgen::prelude::*;

use crate::wasm_accreditations::WasmAccreditations;
use crate::wasm_statement_name::WasmStatementName;
use crate::wasm_statement_value::WasmStatementValue;

/// A client to interact with Hierarchies objects on the IOTA ledger.
///
/// This client is used for read-only operations, meaning it does not require an account
/// or signing capabilities. For write operations, use {@link HierarchiesClient}.
#[derive(Clone)]
#[wasm_bindgen(js_name = HierarchiesClientReadOnly)]
pub struct WasmHierarchiesClientReadOnly(pub(crate) ITHClientReadOnly);

// Builder-related functions
#[wasm_bindgen(js_class = HierarchiesClientReadOnly)]
impl WasmHierarchiesClientReadOnly {
    /// Creates a new instance of `HierarchiesClientReadOnly`.
    ///
    /// # Arguments
    /// * `iota_client` - The IOTA client used for interacting with the ledger.
    ///
    /// # Returns
    /// A new `HierarchiesClientReadOnly` instance.
    #[wasm_bindgen(js_name = create)]
    pub async fn new(iota_client: WasmIotaClient) -> Result<WasmHierarchiesClientReadOnly> {
        let inner_client = ITHClientReadOnly::new(iota_client)
            .await
            .map_err(wasm_error)?;
        Ok(WasmHierarchiesClientReadOnly(inner_client))
    }

    /// Creates a new instance of `HierarchiesClientReadOnly` using a specific package ID.
    ///
    /// # Arguments
    /// * `iota_client` - The IOTA client used for interacting with the ledger.
    /// * `iota_hierarchies_pkg_id` - The hierarchies package ID.
    ///
    /// # Returns
    /// A new `HierarchiesClientReadOnly` instance.
    #[wasm_bindgen(js_name = createWithPkgId)]
    pub async fn new_new_with_pkg_id(
        iota_client: WasmIotaClient,
        iota_hierarchies_pkg_id: String,
    ) -> Result<WasmHierarchiesClientReadOnly> {
        let inner_client = ITHClientReadOnly::new_with_pkg_id(
            iota_client,
            ObjectID::from_str(&iota_hierarchies_pkg_id)
                .map_err(|e| anyhow!("Could not parse iota_hierarchies_pkg_id: {}", e.to_string()))
                .wasm_result()?,
        )
        .await
        .map_err(wasm_error)?;
        Ok(WasmHierarchiesClientReadOnly(inner_client))
    }

    /// Retrieves the package ID of the used hierarchies package.
    ///
    /// # Returns
    /// A string representing the package ID.
    #[wasm_bindgen(js_name = packageId)]
    pub fn package_id(&self) -> String {
        self.0.package_id().to_string()
    }

    /// Retrieves the history of hierarchies package IDs.
    ///
    /// # Returns
    /// An array of strings representing the package history.
    #[wasm_bindgen(js_name = packageHistory)]
    pub fn package_history(&self) -> Vec<String> {
        self.0
            .package_history()
            .into_iter()
            .map(|pkg_id| pkg_id.to_string())
            .collect()
    }

    /// Retrieves the underlying IOTA client used by this client.
    ///
    /// # Returns
    /// The `IotaClient` instance.
    #[wasm_bindgen(js_name = iotaClient)]
    pub fn iota_client(&self) -> WasmIotaClient {
        (*self.0).clone().into_inner()
    }

    /// Retrieves the network identifier associated with this client.
    ///
    /// # Returns
    /// A string representing the network identifier.
    #[wasm_bindgen]
    pub fn network(&self) -> String {
        self.0.network().to_string()
    }

    /// Retrieves the chain ID associated with this client.
    ///
    /// # Returns
    /// A string representing the chain ID.
    #[wasm_bindgen(js_name = chainId)]
    pub fn chain_id(&self) -> String {
        self.0.chain_id().to_string()
    }

    /// Retrieves all statement names registered in the federation.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    ///
    /// # Returns
    /// A `Result` containing the list of statement names or an [`Error`].
    #[wasm_bindgen(js_name = gestStatements)]
    pub async fn get_statements(
        &self,
        federation_id: WasmObjectID,
    ) -> Result<Vec<WasmStatementName>> {
        todo!("Implement get_statements in WasmHierarchiesClientReadOnly");
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
    #[wasm_bindgen(js_name = isStatementInFederation)]
    pub async fn is_statement_in_federation(
        &self,
        federation_id: WasmObjectID,
        statement_name: WasmStatementName,
    ) -> Result<bool> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        self.0
            .is_statement_in_federation(federation_id, statement_name.into())
            .await
            .map_err(wasm_error)
            .wasm_result()
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
    #[wasm_bindgen(js_name = getAccreditationsToAttest)]
    pub async fn get_accreditations_to_attest(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
    ) -> Result<WasmAccreditations> {
        todo!()
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
    #[wasm_bindgen(js_name = isAttester)]
    pub async fn is_attester(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
    ) -> Result<bool> {
        todo!()
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
    #[wasm_bindgen(js_name = getAccreditationsToAccredit)]
    pub async fn get_accreditations_to_accredit(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
    ) -> Result<WasmAccreditations> {
        todo!()
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
    #[wasm_bindgen(js_name = isAccreditor)]
    pub async fn is_accreditor(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
    ) -> Result<bool> {
        todo!()
    }

    /// Validates a statement for a specific user.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    /// * `statement_name`: The name of the statement to validate.
    /// * `statement_value`: The value of the statement to validate.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the statement is valid or an [`Error`].
    #[wasm_bindgen(js_name = validateStatement)]
    pub async fn validate_statement(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
        statement_name: WasmStatementName,
        statement_value: WasmStatementValue,
    ) -> Result<bool> {
        todo!()
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
    #[wasm_bindgen(js_name = validateStatements)]
    pub async fn validate_statements(
        &self,
        federation_id: WasmObjectID,
        entity_id: WasmObjectID,
        statements: js_sys::Array,
    ) -> Result<bool> {
        todo!()
    }
}
