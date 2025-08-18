// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use hierarchies::core::transactions::properties::add_property::AddProperty;
use hierarchies::core::transactions::properties::revoke_property::RevokeProperty;
use hierarchies::core::transactions::{
    AddRootAuthority, CreateAccreditation as CreateAccreditationToAccredit, CreateAccreditationToAttest,
    CreateFederation, ReinstateRootAuthority, RevokeAccreditationToAccredit, RevokeAccreditationToAttest,
    RevokeRootAuthority,
};
use iota_interaction_ts::bindings::{WasmIotaTransactionBlockEffects, WasmIotaTransactionBlockEvents};
use iota_interaction_ts::core_client::WasmCoreClientReadOnly;
use iota_interaction_ts::wasm_error::{Result, wasm_error};
use product_common::bindings::utils::{
    apply_with_events, build_programmable_transaction, parse_wasm_iota_address, parse_wasm_object_id,
};
use product_common::bindings::{WasmIotaAddress, WasmObjectID};
use wasm_bindgen::prelude::*;

use crate::wasm_types::{WasmFederation, WasmProperty, WasmPropertyName};

/// A wrapper for the `CreateFederation` transaction.
#[wasm_bindgen (js_name=CreateFederation, inspectable)]
pub struct WasmCreateFederation(pub(crate) CreateFederation);

#[wasm_bindgen (js_class=CreateFederation)]
impl WasmCreateFederation {
    /// Creates a new instance of `WasmCreateFederation`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(CreateFederation::new())
    }

    /// Builds and returns a programmable transaction for creating a new federation.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this create federation operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// A `WasmFederation` object representing the newly created federation.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<WasmFederation> {
        apply_with_events(self.0, wasm_effects, wasm_events, client).await
    }
}

/// A wrapper for the `AddRootAuthority` transaction.
#[wasm_bindgen(js_name = AddRootAuthority, inspectable)]
pub struct WasmAddRootAuthority(pub(crate) AddRootAuthority);

#[wasm_bindgen(js_class = AddRootAuthority)]
impl WasmAddRootAuthority {
    /// Creates a new instance of `WasmAddRootAuthority`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `account_id` - The ID of the account to add as a root authority.
    /// * `signer_address` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(federation_id: WasmObjectID, account_id: WasmObjectID, signer_address: WasmIotaAddress) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let account_id = parse_wasm_object_id(&account_id)?;
        let signer_address = parse_wasm_iota_address(&signer_address)?;
        Ok(Self(AddRootAuthority::new(federation_id, account_id, signer_address)))
    }

    /// Builds and returns a programmable transaction for adding a root authority.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this add root authority operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}

/// A wrapper for the `RevokeRootAuthority` transaction.
#[wasm_bindgen(js_name = RevokeRootAuthority, inspectable)]
pub struct WasmRevokeRootAuthority(pub(crate) RevokeRootAuthority);

#[wasm_bindgen(js_class = RevokeRootAuthority)]
impl WasmRevokeRootAuthority {
    /// Creates a new instance of `WasmRevokeRootAuthority`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `account_id` - The ID of the account to revoke as a root authority.
    /// * `signer_address` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(federation_id: WasmObjectID, account_id: WasmObjectID, signer_address: WasmIotaAddress) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let account_id = parse_wasm_object_id(&account_id)?;
        let signer_address = parse_wasm_iota_address(&signer_address)?;

        Ok(Self(RevokeRootAuthority::new(
            federation_id,
            account_id,
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for revoking a root authority.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this revoke root authority operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}

/// A wrapper for the `ReinstateRootAuthority` transaction.
#[wasm_bindgen(js_name = ReinstateRootAuthority, inspectable)]
pub struct WasmReinstateRootAuthority(pub(crate) ReinstateRootAuthority);

#[wasm_bindgen(js_class = ReinstateRootAuthority)]
impl WasmReinstateRootAuthority {
    /// Creates a new instance of `WasmReinstateRootAuthority`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `account_id` - The ID of the account to reinstate as a root authority.
    /// * `signer_address` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(federation_id: WasmObjectID, account_id: WasmObjectID, signer_address: WasmIotaAddress) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let account_id = parse_wasm_object_id(&account_id)?;
        let signer_address = parse_wasm_iota_address(&signer_address)?;

        Ok(Self(ReinstateRootAuthority::new(
            federation_id,
            account_id,
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for reinstating a root authority.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this reinstate root authority operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}

/// A wrapper for the `AddProperty` transaction.
#[wasm_bindgen(js_name = AddProperty, inspectable)]
pub struct WasmAddProperty(pub(crate) AddProperty);

#[wasm_bindgen(js_class = AddProperty)]
impl WasmAddProperty {
    /// Creates a new instance of `WasmAddProperty`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `property` - The property to add.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(federation_id: WasmObjectID, property: &WasmProperty, owner: WasmIotaAddress) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let signer_address = parse_wasm_iota_address(&owner)?;

        Ok(Self(AddProperty::new(
            federation_id,
            property.clone().into(),
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for adding a property.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this add property operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}

/// A wrapper for the `RevokeProperty` transaction.
#[wasm_bindgen(js_name = RevokeProperty, inspectable)]
pub struct WasmRevokeProperty(pub(crate) RevokeProperty);

#[wasm_bindgen(js_class = RevokeProperty)]
impl WasmRevokeProperty {
    /// Creates a new instance of `WasmRevokeProperty`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `property_name` - The name of the property to revoke.
    /// * `valid_to_ms` - The timestamp until which the property is valid.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        property_name: WasmPropertyName,
        valid_to_ms: Option<u64>,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let property_name = property_name.into();
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(RevokeProperty::new(
            federation_id,
            property_name,
            valid_to_ms,
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for revoking a property.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this revoke property operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}

/// A wrapper for the `CreateAccreditationToAttest` transaction.
#[wasm_bindgen(js_name = CreateAccreditationToAttest, inspectable)]
pub struct WasmCreateAccreditationToAttest(pub(crate) CreateAccreditationToAttest);

#[wasm_bindgen(js_class = CreateAccreditationToAttest)]
impl WasmCreateAccreditationToAttest {
    /// Creates a new instance of `WasmCreateAccreditationToAttest`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `receiver` - The ID of the receiver of the accreditation.
    /// * `want_properties` - The properties for which permissions are being granted.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        receiver: WasmObjectID,
        want_properties: js_sys::Array,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let receiver = parse_wasm_object_id(&receiver)?;
        let want_properties = want_properties
            .iter()
            .map(|v| serde_wasm_bindgen::from_value::<WasmProperty>(v).map_err(wasm_error))
            .collect::<Result<Vec<_>>>()?;
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(CreateAccreditationToAttest::new(
            federation_id,
            receiver,
            want_properties.into_iter().map(|s| s.into()),
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for creating an accreditation to accredit.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this create accreditation to accredit operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}

/// A wrapper for the `RevokeAccreditationToAccredit` transaction.
#[wasm_bindgen(js_name = RevokeAccreditationToAttest, inspectable)]
pub struct WasmRevokeAccreditationToAttest(pub(crate) RevokeAccreditationToAttest);

#[wasm_bindgen(js_class = RevokeAccreditationToAttest)]
impl WasmRevokeAccreditationToAttest {
    /// Creates a new instance of `WasmRevokeAccreditationToAttest`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `entity_id` - The ID of the user whose accreditation is being revoked.
    /// * `accreditation_id` - The ID of the accreditation to revoke.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        entity_id: WasmObjectID,
        accreditation_id: WasmObjectID,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let entity_id = parse_wasm_object_id(&entity_id)?;
        let accreditation_id = parse_wasm_object_id(&accreditation_id)?;
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(RevokeAccreditationToAttest::new(
            federation_id,
            entity_id,
            accreditation_id,
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for revoking an accreditation to attest.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this revoke accreditation to attest operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}

/// A wrapper for the `CreateAccreditationToAccredit` transaction.
#[wasm_bindgen(js_name = CreateAccreditationToAccredit, inspectable)]
pub struct WasmCreateAccreditationToAccredit(pub(crate) CreateAccreditationToAccredit);

#[wasm_bindgen(js_class = CreateAccreditationToAccredit)]
impl WasmCreateAccreditationToAccredit {
    /// Creates a new instance of `WasmCreateAccreditationToAccredit`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `receiver_id` - The ID of the receiver of the accreditation.
    /// * `want_properties` - The properties for which permissions are being granted.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        receiver_id: WasmObjectID,
        want_properties: js_sys::Array,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let receiver_id = parse_wasm_object_id(&receiver_id)?;
        let want_properties = want_properties
            .iter()
            .map(|v| serde_wasm_bindgen::from_value::<WasmProperty>(v).map_err(wasm_error))
            .collect::<Result<Vec<_>>>()?;
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(CreateAccreditationToAccredit::new(
            federation_id,
            receiver_id,
            want_properties.into_iter().map(|s| s.into()).collect(),
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for creating an accreditation to accredit.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this create accreditation to accredit operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}

/// A wrapper for the `RevokeAccreditationToAccredit` transaction.
#[wasm_bindgen(js_name = RevokeAccreditationToAccredit, inspectable)]
pub struct WasmRevokeAccreditationToAccredit(pub(crate) RevokeAccreditationToAccredit);

#[wasm_bindgen(js_class = RevokeAccreditationToAccredit)]
impl WasmRevokeAccreditationToAccredit {
    /// Creates a new instance of `WasmRevokeAccreditationToAccredit`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `entity_id` - The ID of entity whose accreditation is being revoked.
    /// * `accreditation_id` - The ID of the accreditation to revoke.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        entity_id: WasmObjectID,
        accreditation_id: WasmObjectID,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let entity_id = parse_wasm_object_id(&entity_id)?;
        let accreditation_id = parse_wasm_object_id(&accreditation_id)?;
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(RevokeAccreditationToAccredit::new(
            federation_id,
            entity_id,
            accreditation_id,
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for revoking an accreditation to accredit.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    ///
    /// # Returns
    ///
    /// The binary BCS serialization of the programmable transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be built.
    #[wasm_bindgen(js_name = buildProgrammableTransaction)]
    pub async fn build_programmable_transaction(&self, client: &WasmCoreClientReadOnly) -> Result<Vec<u8>> {
        build_programmable_transaction(&self.0, client).await
    }

    /// Applies transaction effects and events to this revoke accreditation to accredit operation.
    ///
    /// # Arguments
    ///
    /// * `effects` - The transaction block effects to apply.
    /// * `events` - The transaction block events to apply.
    /// * `client` - A read-only client for blockchain interaction.
    #[wasm_bindgen(js_name = applyWithEvents)]
    pub async fn apply_with_events(
        self,
        wasm_effects: &WasmIotaTransactionBlockEffects,
        wasm_events: &WasmIotaTransactionBlockEvents,
        client: &WasmCoreClientReadOnly,
    ) -> Result<()> {
        apply_with_events(self.0, wasm_effects, wasm_events, client)
            .await
            .map_err(wasm_error)
    }
}
