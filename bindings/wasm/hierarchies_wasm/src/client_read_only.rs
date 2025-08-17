// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::str::FromStr;

use anyhow::anyhow;
use hierarchies::client::HierarchiesClientReadOnly;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction_ts::bindings::WasmIotaClient;
use iota_interaction_ts::wasm_error::{Result, WasmResult, wasm_error};
use product_common::bindings::WasmObjectID;
use product_common::bindings::utils::parse_wasm_object_id;
use product_common::core_client::CoreClientReadOnly;
use wasm_bindgen::prelude::*;

use crate::wasm_types::{WasmAccreditations, WasmFederation, WasmPropertyName, WasmPropertyValue};

/// A client to interact with Hierarchies objects on the IOTA ledger.
///
/// This client is used for read-only operations, meaning it does not require an account
/// or signing capabilities. For write operations, use {@link HierarchiesClient}.
#[derive(Clone)]
#[wasm_bindgen(js_name = HierarchiesClientReadOnly)]
pub struct WasmHierarchiesClientReadOnly(pub(crate) HierarchiesClientReadOnly);

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
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with `WasmHierarchiesClientReadOnly`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const client = await HierarchiesClientReadOnly.create(iotaClient);
    ///   console.log("Client created:", client);
    /// } catch (error) {
    ///   console.error("Failed to create client:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = create)]
    pub async fn new(iota_client: WasmIotaClient) -> Result<WasmHierarchiesClientReadOnly> {
        let inner_client = HierarchiesClientReadOnly::new(iota_client).await.map_err(wasm_error)?;
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
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with `WasmHierarchiesClientReadOnly`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const client = await HierarchiesClientReadOnly.createWithPkgId(iotaClient, pkgId);
    ///   console.log("Client created:", client);
    /// } catch (error) {
    ///   console.error("Failed to create client:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = createWithPkgId)]
    pub async fn new_new_with_pkg_id(
        iota_client: WasmIotaClient,
        iota_hierarchies_pkg_id: String,
    ) -> Result<WasmHierarchiesClientReadOnly> {
        let inner_client = HierarchiesClientReadOnly::new_with_pkg_id(
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

    /// Retrieves a federation by its ID.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    ///
    /// # Returns
    /// A `Result` containing the [`Federation`] object or an [`Error`].
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with `WasmFederation`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const federation = await client.getFederationById(federationId);
    ///   console.log("Federation:", federation);
    /// } catch (error) {
    ///   console.error("Failed to get federation:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = getFederationById)]
    pub async fn get_federation_by_id(&self, federation_id: WasmObjectID) -> Result<WasmFederation> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let federation = self.0.get_federation_by_id(federation_id).await.map_err(wasm_error)?;
        Ok(federation.into())
    }

    /// Check if root authority is in the federation.

    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the user is a root authority or an [`Error`].
    #[wasm_bindgen(js_name = isRootAuthority)]
    pub async fn is_root_authority(&self, federation_id: WasmObjectID, user_id: WasmObjectID) -> Result<bool> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let is_root_authority = self
            .0
            .is_root_authority(federation_id, user_id)
            .await
            .map_err(wasm_error)?;
        Ok(is_root_authority)
    }

    /// Retrieves all property names registered in the federation.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    ///
    /// # Returns
    /// A `Result` containing the list of property names or an [`Error`].
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with `WasmProperty[]`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const properties = await client.getProperties(federationId);
    ///   console.log("Properties:", properties);
    /// } catch (error) {
    ///   console.error("Failed to get properties:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = getProperties)]
    pub async fn get_properties(&self, federation_id: WasmObjectID) -> Result<Vec<WasmPropertyName>> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let properties = self.0.get_properties(federation_id).await.map_err(wasm_error)?;
        Ok(properties.into_iter().map(|property| property.into()).collect())
    }

    /// Checks if a property is registered in the federation.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `property_name`: The name of the property to check.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the property is registered or an [`Error`].
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with a `boolean`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const isRegistered = await client.isPropertyInFederation(federationId, propertyName);
    ///   console.log("Is property registered:", isRegistered);
    /// } catch (error) {
    ///   console.error("Failed to check property registration:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = isPropertyInFederation)]
    pub async fn is_property_in_federation(
        &self,
        federation_id: WasmObjectID,
        property_name: WasmPropertyName,
    ) -> Result<bool> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        self.0
            .is_property_in_federation(federation_id, property_name.into())
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
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with `WasmAccreditations`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const accreditations = await client.getAccreditationsToAttest(federationId, userId);
    ///   console.log("Accreditations:", accreditations);
    /// } catch (error) {
    ///   console.error("Failed to get accreditations:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = getAccreditationsToAttest)]
    pub async fn get_accreditations_to_attest(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
    ) -> Result<WasmAccreditations> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let accreditations = self
            .0
            .get_accreditations_to_attest(federation_id, user_id)
            .await
            .map_err(wasm_error)?;
        Ok(accreditations.into())
    }

    /// Checks if a user has attestation accreditation.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the user has attestation accreditation or an [`Error`].
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with a `boolean`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const isAttester = await client.isAttester(federationId, userId);
    ///   console.log("Is attester:", isAttester);
    /// } catch (error) {
    ///   console.error("Failed to check attester status:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = isAttester)]
    pub async fn is_attester(&self, federation_id: WasmObjectID, user_id: WasmObjectID) -> Result<bool> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let is_attester = self.0.is_attester(federation_id, user_id).await.map_err(wasm_error)?;
        Ok(is_attester)
    }

    /// Retrieves accreditations to accredit for a specific user.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    ///
    /// # Returns
    /// A `Result` containing the accreditations to accredit for the user or an [`Error`].
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with `WasmAccreditations`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const accreditations = await client.getAccreditationsToAccredit(federationId, userId);
    ///   console.log("Accreditations:", accreditations);
    /// } catch (error) {
    ///   console.error("Failed to get accreditations:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = getAccreditationsToAccredit)]
    pub async fn get_accreditations_to_accredit(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
    ) -> Result<WasmAccreditations> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let accreditations = self
            .0
            .get_accreditations_to_accredit(federation_id, user_id)
            .await
            .map_err(wasm_error)?;
        Ok(accreditations.into())
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
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with a `boolean`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const isAccreditor = await client.isAccreditor(federationId, userId);
    ///   console.log("Is accreditor:", isAccreditor);
    /// } catch (error) {
    ///   console.error("Failed to check accreditor status:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = isAccreditor)]
    pub async fn is_accreditor(&self, federation_id: WasmObjectID, user_id: WasmObjectID) -> Result<bool> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let is_accreditor = self.0.is_accreditor(federation_id, user_id).await.map_err(wasm_error)?;
        Ok(is_accreditor)
    }

    /// Validates a property for a specific user.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    /// * `property_name`: The name of the property to validate.
    /// * `property_value`: The value of the property to validate.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the property is valid or an [`Error`].
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with a `boolean`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const isValid = await client.validateProperty(federationId, userId, propertyName, propertyValue);
    ///   console.log("Is property valid:", isValid);
    /// } catch (error) {
    ///   console.error("Failed to validate property:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = validateProperty)]
    pub async fn validate_property(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
        property_name: WasmPropertyName,
        property_value: WasmPropertyValue,
    ) -> Result<bool> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let property_name = property_name.into();
        let property_value = property_value.into();
        let is_valid = self
            .0
            .validate_property(federation_id, user_id, property_name, property_value)
            .await
            .map_err(wasm_error)?;
        Ok(is_valid)
    }

    /// Validates multiple properties for a specific user.
    ///
    /// # Arguments
    ///
    /// * `federation_id`: The [`ObjectID`] of the federation.
    /// * `user_id`: The [`ObjectID`] of the user.
    /// * `properties`: The properties to validate.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating if the properties are valid or an [`Error`].
    ///
    /// # TypeScript Usage
    /// This method returns a `Promise` in TypeScript.
    /// - On success, the promise resolves with a `boolean`.
    /// - On failure, the promise rejects with an `Error`.
    ///
    /// ```typescript
    /// try {
    ///   const areValid = await client.validateProperties(federationId, userId, properties);
    ///   console.log("Are properties valid:", areValid);
    /// } catch (error) {
    ///   console.error("Failed to validate properties:", error);
    /// }
    /// ```
    #[wasm_bindgen(js_name = validateProperties)]
    pub async fn validate_properties(
        &self,
        federation_id: WasmObjectID,
        entity_id: WasmObjectID,
        properties: js_sys::Map,
    ) -> Result<bool> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let entity_id = parse_wasm_object_id(&entity_id)?;

        let mut converted_properties = HashMap::new();

        properties.for_each(&mut |value, key| {
            if let (Ok(property_name), Ok(property_value)) = (
                serde_wasm_bindgen::from_value::<WasmPropertyName>(key),
                serde_wasm_bindgen::from_value::<WasmPropertyValue>(value),
            ) {
                converted_properties.insert(property_name.into(), property_value.into());
            }
        });

        let is_valid = self
            .0
            .validate_properties(federation_id, entity_id, converted_properties)
            .await
            .map_err(wasm_error)?;
        Ok(is_valid)
    }
}
