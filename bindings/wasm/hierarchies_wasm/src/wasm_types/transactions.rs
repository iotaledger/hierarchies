use std::collections::HashSet;

use iota_interaction_ts::bindings::{WasmIotaTransactionBlockEffects, WasmIotaTransactionBlockEvents};
use iota_interaction_ts::core_client::WasmCoreClientReadOnly;
use iota_interaction_ts::wasm_error::{wasm_error, Result};
use ith::core::transactions::statements::add_statement::AddStatement;
use ith::core::transactions::statements::revoke_statement::RevokeStatement;
use ith::core::transactions::{
    AddRootAuthority, CreateAccreditation, CreateAccreditationToAttest, CreateFederation,
    RevokeAccreditationToAccredit, RevokeAccreditationToAttest,
};
use product_common::bindings::utils::{
    apply_with_events, build_programmable_transaction, parse_wasm_iota_address, parse_wasm_object_id,
};
use product_common::bindings::{WasmIotaAddress, WasmObjectID};
use wasm_bindgen::prelude::*;

use crate::wasm_types::{WasmFederation, WasmStatement, WasmStatementName, WasmStatementValue};

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

/// A wrapper for the `AddStatement` transaction.
#[wasm_bindgen(js_name = AddStatement, inspectable)]
pub struct WasmAddStatement(pub(crate) AddStatement);

#[wasm_bindgen(js_class = AddStatement)]
impl WasmAddStatement {
    /// Creates a new instance of `WasmAddStatement`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `statement_name` - The name of the statement.
    /// * `allowed_values` - The allowed values for the statement.
    /// * `allow_any` - A flag indicating if any value is allowed.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        statement_name: WasmStatementName,
        allowed_values: Vec<WasmStatementValue>,
        allow_any: bool,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let statement_name = statement_name.into();
        let allowed_values = allowed_values.into_iter().map(|v| v.into()).collect::<HashSet<_>>();
        let signer_address = parse_wasm_iota_address(&owner)?;

        Ok(Self(AddStatement::new(
            federation_id,
            statement_name,
            allowed_values,
            allow_any,
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for adding a statement.
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

    /// Applies transaction effects and events to this add statement operation.
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

/// A wrapper for the `RevokeStatement` transaction.
#[wasm_bindgen(js_name = RevokeStatement, inspectable)]
pub struct WasmRevokeStatement(pub(crate) RevokeStatement);

#[wasm_bindgen(js_class = RevokeStatement)]
impl WasmRevokeStatement {
    /// Creates a new instance of `WasmRevokeStatement`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `statement_name` - The name of the statement to revoke.
    /// * `valid_to_ms` - The timestamp until which the statement is valid.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        statement_name: WasmStatementName,
        valid_to_ms: Option<u64>,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let statement_name = statement_name.into();
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(RevokeStatement::new(
            federation_id,
            statement_name,
            valid_to_ms,
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for revoking a statement.
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

    /// Applies transaction effects and events to this revoke statement operation.
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
    /// * `want_statements` - The statements for which permissions are being granted.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        receiver: WasmObjectID,
        want_statements: js_sys::Array,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let receiver = parse_wasm_object_id(&receiver)?;
        let want_statements = want_statements
            .iter()
            .map(|v| serde_wasm_bindgen::from_value::<WasmStatement>(v).map_err(wasm_error))
            .collect::<Result<Vec<_>>>()?;
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(CreateAccreditationToAttest::new(
            federation_id,
            receiver,
            want_statements.into_iter().map(|s| s.into()),
            signer_address,
        )))
    }

    /// Builds and returns a programmable transaction for creating an accreditation to attest.
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

    /// Applies transaction effects and events to this create accreditation to attest operation.
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

/// A wrapper for the `RevokeAccreditationToAttest` transaction.
#[wasm_bindgen(js_name = RevokeAccreditationToAttest, inspectable)]
pub struct WasmRevokeAccreditationToAttest(pub(crate) RevokeAccreditationToAttest);

#[wasm_bindgen(js_class = RevokeAccreditationToAttest)]
impl WasmRevokeAccreditationToAttest {
    /// Creates a new instance of `WasmRevokeAccreditationToAttest`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `receiver` - The ID of the receiver of the accreditation.
    /// * `accreditation_id` - The ID of the accreditation to revoke.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        receiver: WasmObjectID,
        accreditation_id: WasmObjectID,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let receiver = parse_wasm_object_id(&receiver)?;
        let accreditation_id = parse_wasm_object_id(&accreditation_id)?;
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(RevokeAccreditationToAttest::new(
            federation_id,
            receiver,
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
pub struct WasmCreateAccreditationToAccredit(pub(crate) CreateAccreditation);

#[wasm_bindgen(js_class = CreateAccreditationToAccredit)]
impl WasmCreateAccreditationToAccredit {
    /// Creates a new instance of `WasmCreateAccreditationToAccredit`.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The ID of the federation.
    /// * `receiver` - The ID of the receiver of the accreditation.
    /// * `want_statements` - The statements for which permissions are being granted.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        receiver: WasmObjectID,
        want_statements: js_sys::Array,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let receiver = parse_wasm_object_id(&receiver)?;
        let want_statements = want_statements
            .iter()
            .map(|v| serde_wasm_bindgen::from_value::<WasmStatement>(v).map_err(wasm_error))
            .collect::<Result<Vec<_>>>()?;
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(CreateAccreditation::new(
            federation_id,
            receiver,
            want_statements.into_iter().map(|s| s.into()).collect(),
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
    /// * `user_id` - The ID of the user whose accreditation is being revoked.
    /// * `permission_id` - The ID of the permission to revoke.
    /// * `owner` - The address of the transaction signer.
    #[wasm_bindgen(constructor)]
    pub fn new(
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
        permission_id: WasmObjectID,
        owner: WasmIotaAddress,
    ) -> Result<Self> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let permission_id = parse_wasm_object_id(&permission_id)?;
        let signer_address = parse_wasm_iota_address(&owner)?;
        Ok(Self(RevokeAccreditationToAccredit::new(
            federation_id,
            user_id,
            permission_id,
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
