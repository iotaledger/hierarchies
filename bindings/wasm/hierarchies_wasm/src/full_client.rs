// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use iota_interaction_ts::bindings::{WasmIotaClient, WasmTransactionSigner};
use iota_interaction_ts::wasm_error::{Result, WasmResult};
use iota_interaction_ts::WasmPublicKey;
use hierarchies::client::HierarchiesClient;
use hierarchies::core::types::statements::name::StatementName;
use hierarchies::core::types::statements::value::StatementValue;
use product_common::bindings::transaction::WasmTransactionBuilder;
use product_common::bindings::utils::{into_transaction_builder, parse_wasm_object_id};
use product_common::bindings::{WasmIotaAddress, WasmObjectID};
use product_common::core_client::{CoreClient, CoreClientReadOnly};
use wasm_bindgen::prelude::*;

use crate::client_read_only::WasmHierarchiesClientReadOnly;
use crate::wasm_types::transactions::{
    WasmAddRootAuthority, WasmAddStatement, WasmCreateAccreditationToAccredit, WasmCreateAccreditationToAttest,
    WasmCreateFederation, WasmRevokeAccreditationToAccredit, WasmRevokeAccreditationToAttest, WasmRevokeStatement,
    WasmRevokeRootAuthority,
};
use crate::wasm_types::{WasmStatement, WasmStatementName, WasmStatementValue};

/// A client to interact with Hierarchies objects on the IOTA ledger.
///
/// This client is used for read and write operations. For read-only capabilities,
/// you can use {@link HierarchiesClientReadOnly}, which does not require an account or signing capabilities.

#[wasm_bindgen(js_name = HierarchiesClient)]
pub struct WasmHierarchiesClient(pub(crate) HierarchiesClient<WasmTransactionSigner>);

#[wasm_bindgen(js_class=HierarchiesClient)]
impl WasmHierarchiesClient {
    /// Creates a new client with signing capabilities.
    ///
    /// # Arguments
    ///
    /// * `client` - A read-only client for blockchain interaction.
    /// * `signer` - A signer for transaction authorization.
    ///
    /// # Errors
    ///
    /// Returns an error if the signer's public key cannot be retrieved.
    ///
    /// ```
    #[wasm_bindgen(constructor)]
    pub async fn new(
        client: WasmHierarchiesClientReadOnly,
        signer: WasmTransactionSigner,
    ) -> Result<WasmHierarchiesClient> {
        let inner_client = HierarchiesClient::new(client.0, signer).await.wasm_result()?;
        Ok(WasmHierarchiesClient(inner_client))
    }

    /// Creates a new [`WasmTransactionBuilder`] for creating a new federation.
    ///
    /// See [`HierarchiesClient::create_new_federation`] for more details.
    #[wasm_bindgen(js_name = createNewFederation)]
    pub fn create_new_federation(&self) -> Result<WasmTransactionBuilder> {
        let tx = self.0.create_new_federation().into_inner();

        Ok(into_transaction_builder(WasmCreateFederation(tx)))
    }

    /// Creates a [`WasmTransactionBuilder`] for adding a root authority to a federation.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The [`WasmObjectID`] of the federation.
    /// * `account_id` - The [`WasmObjectID`] of the account to add as a root authority.
    #[wasm_bindgen(js_name = addRootAuthority)]
    pub fn add_root_authority(
        &self,
        federation_id: WasmObjectID,
        account_id: WasmObjectID,
    ) -> Result<WasmTransactionBuilder> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let account_id = parse_wasm_object_id(&account_id)?;

        let tx = self.0.add_root_authority(federation_id, account_id).into_inner();
        Ok(into_transaction_builder(WasmAddRootAuthority(tx)))
    }

    /// Creates a [`WasmTransactionBuilder`] for revoking a root authority from a federation.
    ///
    /// Only existing root authorities can revoke other root authorities.
    /// Cannot revoke the last root authority to prevent lockout.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The [`WasmObjectID`] of the federation.
    /// * `account_id` - The [`WasmObjectID`] of the account to revoke as a root authority.
    #[wasm_bindgen(js_name = revokeRootAuthority)]
    pub fn revoke_root_authority(
        &self,
        federation_id: WasmObjectID,
        account_id: WasmObjectID,
    ) -> Result<WasmTransactionBuilder> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let account_id = parse_wasm_object_id(&account_id)?;

        let tx = self.0.revoke_root_authority(federation_id, account_id).into_inner();
        Ok(into_transaction_builder(WasmRevokeRootAuthority(tx)))
    }

    /// Creates a new [`WasmTransactionBuilder`] for adding a statement to a federation.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The [`WasmObjectID`] of the federation.
    /// * `statement_name` - The name of the statement.
    /// * `allowed_values` - The allowed values for the statement.
    /// * `allow_any` - Whether to allow any value.
    #[wasm_bindgen(js_name = addStatement)]
    pub fn add_statement(
        &self,
        federation_id: WasmObjectID,
        statement_name: &WasmStatementName,
        allowed_values: Box<[WasmStatementValue]>,
        allow_any: bool,
    ) -> Result<WasmTransactionBuilder> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let statement_name = StatementName::from(statement_name.0.clone());

        let unique_allowed_values: HashSet<StatementValue> =
            HashSet::from_iter(allowed_values.iter().cloned().map(|v| v.0.clone()));

        let tx = self
            .0
            .add_statement(federation_id, statement_name, unique_allowed_values, allow_any)
            .into_inner();

        Ok(into_transaction_builder(WasmAddStatement(tx)))
    }

    /// Creates a new [`WasmTransactionBuilder`] for revoking a statement from a federation.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The [`WasmObjectID`] of the federation.
    /// * `statement_name` - The name of the statement to revoke.
    /// * `valid_to_ms` - The timestamp in milliseconds until which the statement is valid.
    pub fn revoke_statement(
        &self,
        federation_id: WasmObjectID,
        statement_name: &WasmStatementName,
        valid_to_ms: Option<u64>,
    ) -> Result<WasmTransactionBuilder> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let statement_name = StatementName::from(statement_name.0.clone());
        let tx = self
            .0
            .revoke_statement(federation_id, statement_name, valid_to_ms)
            .into_inner();
        Ok(into_transaction_builder(WasmRevokeStatement(tx)))
    }

    /// Creates a new [`WasmTransactionBuilder`] for creating an accreditation to attest.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The [`WasmObjectID`] of the federation.
    /// * `receiver` - The [`WasmObjectID`] of the receiver of the accreditation.
    /// * `want_statements` - The statements for which permissions are being granted.
    #[wasm_bindgen(js_name = createAccreditationToAttest)]
    pub fn create_accreditation_to_attest(
        &self,
        federation_id: WasmObjectID,
        receiver: WasmObjectID,
        want_statements: Box<[WasmStatement]>,
    ) -> Result<WasmTransactionBuilder> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let receiver = parse_wasm_object_id(&receiver)?;

        let tx = self
            .0
            .create_accreditation_to_attest(
                federation_id,
                receiver,
                want_statements.iter().cloned().map(|s| s.into()),
            )
            .into_inner();

        Ok(into_transaction_builder(WasmCreateAccreditationToAttest(tx)))
    }

    /// Creates a new [`WasmTransactionBuilder`] for revoking an accreditation to attest.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The [`WasmObjectID`] of the federation.
    /// * `user_id` - The [`WasmObjectID`] of the user whose accreditation is being revoked.
    /// * `permission_id` - The [`WasmObjectID`] of the permission to revoke.
    #[wasm_bindgen(js_name = revokeAccreditationToAttest)]
    pub fn revoke_accreditation_to_attest(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
        permission_id: WasmObjectID,
    ) -> Result<WasmTransactionBuilder> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let permission_id = parse_wasm_object_id(&permission_id)?;

        let tx = self
            .0
            .revoke_accreditation_to_attest(federation_id, user_id, permission_id)
            .into_inner();

        Ok(into_transaction_builder(WasmRevokeAccreditationToAttest(tx)))
    }

    /// Creates a new [`WasmTransactionBuilder`] for creating an accreditation to accredit.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The [`WasmObjectID`] of the federation.
    /// * `receiver` - The [`WasmObjectID`] of the receiver of the accreditation.
    /// * `want_statements` - The statements for which permissions are being granted.
    #[wasm_bindgen(js_name = createAccreditationToAccredit)]
    pub fn create_accreditation_to_accredit(
        &self,
        federation_id: WasmObjectID,
        receiver: WasmObjectID,
        want_statements: Box<[WasmStatement]>,
    ) -> Result<WasmTransactionBuilder> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let receiver = parse_wasm_object_id(&receiver)?;

        let tx = self
            .0
            .create_accreditation_to_accredit(
                federation_id,
                receiver,
                want_statements.iter().cloned().map(|s| s.into()),
            )
            .into_inner();

        Ok(into_transaction_builder(WasmCreateAccreditationToAccredit(tx)))
    }

    /// Creates a new [`WasmTransactionBuilder`] for revoking an accreditation to accredit.
    ///
    /// # Arguments
    ///
    /// * `federation_id` - The [`WasmObjectID`] of the federation.
    /// * `user_id` - The [`WasmObjectID`] of the user whose accreditation is being revoked.
    /// * `accreditation_id` - The [`WasmObjectID`] of the accreditation to revoke.
    #[wasm_bindgen(js_name = revokeAccreditationToAccredit)]
    pub fn revoke_accreditation_to_accredit(
        &self,
        federation_id: WasmObjectID,
        user_id: WasmObjectID,
        accreditation_id: WasmObjectID,
    ) -> Result<WasmTransactionBuilder> {
        let federation_id = parse_wasm_object_id(&federation_id)?;
        let user_id = parse_wasm_object_id(&user_id)?;
        let accreditation_id = parse_wasm_object_id(&accreditation_id)?;

        let tx = self
            .0
            .revoke_accreditation_to_accredit(federation_id, user_id, accreditation_id)
            .into_inner();
        Ok(into_transaction_builder(WasmRevokeAccreditationToAccredit(tx)))
    }

    /// Retrieves the sender's public key.
    #[wasm_bindgen(js_name = senderPublicKey)]
    pub fn sender_public_key(&self) -> Result<WasmPublicKey> {
        self.0.sender_public_key().try_into()
    }

    /// Retrieves the sender's address.
    #[wasm_bindgen(js_name = senderAddress)]
    pub fn sender_address(&self) -> WasmIotaAddress {
        self.0.sender_address().to_string()
    }

    /// Retrieves the network identifier.
    #[wasm_bindgen(js_name = network)]
    pub fn network(&self) -> String {
        self.0.network().to_string()
    }

    /// Retrieves the package ID.
    #[wasm_bindgen(js_name = packageId)]
    pub fn package_id(&self) -> String {
        self.0.package_id().to_string()
    }

    /// Retrieves the package history.
    #[wasm_bindgen(js_name = packageHistory)]
    pub fn package_history(&self) -> Vec<String> {
        self.0
            .package_history()
            .into_iter()
            .map(|pkg_id| pkg_id.to_string())
            .collect()
    }

    /// Retrieves the IOTA client instance.
    #[wasm_bindgen(js_name = iotaClient)]
    pub fn iota_client(&self) -> WasmIotaClient {
        (**self.0).clone().into_inner()
    }

    /// Retrieves the transaction signer.
    #[wasm_bindgen]
    pub fn signer(&self) -> WasmTransactionSigner {
        self.0.signer().clone()
    }

    /// Retrieves a read-only version of the hierarchies client.
    #[wasm_bindgen(js_name = readOnly)]
    pub fn read_only(&self) -> WasmHierarchiesClientReadOnly {
        WasmHierarchiesClientReadOnly((*self.0).clone())
    }
}
