use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use fastcrypto::hash::HashFunction;
use fastcrypto::traits::ToFromBytes;
use iota_sdk::rpc_types::{
  IotaExecutionStatus, IotaTransactionBlockEffects, IotaTransactionBlockEffectsV1, IotaTransactionBlockResponse,
  IotaTransactionBlockResponseOptions,
};
use iota_sdk::types::base_types::{IotaAddress, ObjectID};
use iota_sdk::types::crypto::{DefaultHash, Signature, SignatureScheme};
use iota_sdk::types::id::ID;
use iota_sdk::types::quorum_driver_types::ExecuteTransactionRequestType;
use iota_sdk::types::transaction::{ProgrammableTransaction, Transaction, TransactionData};
use secret_storage::Signer;
use shared_crypto::intent::{Intent, IntentMessage};

use super::HTFClientReadOnly;
use crate::federation;
use crate::key::{IotaKeySignature, SigningInfo};
use crate::types::trusted_constraints::TrustedPropertyConstraints;
use crate::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use crate::types::Federation;
use crate::utils::convert_to_address;

/// The `HTFClient` struct is responsible for managing the connection to the
/// IOTA network and
/// executing transactions on behalf of the HTF (Hierarchial Trust Framework) package.
pub struct HTFClient<S> {
  read_client: HTFClientReadOnly,
  signer: S,
  gas_budget: u64,
  signing_info: SigningInfo,
}

impl<S> HTFClient<S>
where
  S: Signer<IotaKeySignature>,
{
  /// Creates a new [`HTFClient`] instance.
  ///
  /// This function initializes an `HTFClient` with the provided [`HTFClientReadOnly`], `Signer`, and gas budget.
  /// The `SigningInfo` struct is also initialized with the signer's public key and the derived sender address.
  ///
  /// # Arguments
  /// * `read_client` - The [`HTFClientReadOnly`] instance to use for read operations.
  /// * `signer` - The `Signer` instance to use for signing transactions.
  /// * `gas_budget` - The gas budget to use for transactions.
  ///
  /// # Returns
  /// A new `HTFClient` instance.
  pub async fn new(read_client: HTFClientReadOnly, signer: S, gas_budget: u64) -> anyhow::Result<Self> {
    let pub_key = signer.public_key().await?;
    let address = convert_to_address(&pub_key)?;
    let info = SigningInfo {
      sender_address: address,
      sender_public_key: pub_key,
    };

    Ok(Self {
      read_client,
      signer,
      signing_info: info,
      gas_budget,
    })
  }

  /// Returns the sender's address.
  pub fn sender_address(&self) -> IotaAddress {
    self.signing_info.sender_address
  }

  pub fn sender_public_key(&self) -> &[u8] {
    &self.signing_info.sender_public_key
  }

  pub(crate) async fn execute_transaction(
    &self,
    tx: ProgrammableTransaction,
  ) -> anyhow::Result<IotaTransactionBlockResponse> {
    let tx_data = self.get_transaction_data(tx).await?;
    let kinesis_signature = self.sign_transaction_data(&tx_data).await?;

    // execute tx
    let response = self
      .quorum_driver_api()
      .execute_transaction_block(
        Transaction::from_data(tx_data, vec![kinesis_signature]),
        IotaTransactionBlockResponseOptions::full_content(),
        Some(ExecuteTransactionRequestType::WaitForLocalExecution),
      )
      .await
      .map_err(anyhow::Error::from)?;

    if let Some(IotaTransactionBlockEffects::V1(IotaTransactionBlockEffectsV1 {
      status: IotaExecutionStatus::Failure { error },
      ..
    })) = &response.effects
    {
      anyhow::bail!("Transaction failed: {}", error);
    }

    Ok(response)
  }

  async fn get_transaction_data(
    &self,
    programmable_transaction: ProgrammableTransaction,
  ) -> anyhow::Result<TransactionData> {
    let gas_price = self.read_client.read_api().get_reference_gas_price().await?;

    let sender = self.sender_address();

    let coin = self.get_coin_for_transaction().await?;
    let tx_data = TransactionData::new_programmable(
      sender,
      vec![coin.object_ref()],
      programmable_transaction,
      self.gas_budget,
      gas_price,
    );

    Ok(tx_data)
  }

  async fn get_coin_for_transaction(&self) -> anyhow::Result<iota_sdk::rpc_types::Coin> {
    let address = self.signing_info.sender_address;

    let coins = self
      .read_client
      .coin_read_api()
      .get_coins(address, None, None, None)
      .await?;

    coins
      .data
      .into_iter()
      .next()
      .ok_or_else(|| anyhow::anyhow!("No coins found for sender address"))
  }

  async fn sign_transaction_data(&self, tx_data: &TransactionData) -> anyhow::Result<Signature> {
    let SigningInfo { sender_public_key, .. } = &self.signing_info;

    let intent = Intent::iota_transaction();
    let intent_msg = IntentMessage::new(intent, tx_data);
    let mut hasher = DefaultHash::default();
    hasher.update(bcs::to_bytes(&intent_msg)?);
    let digest = hasher.finalize().digest;

    let raw_signature = self.signer.sign(&digest).await?;

    let binding = [
      [SignatureScheme::ED25519.flag()].as_slice(),
      &raw_signature,
      sender_public_key,
    ]
    .concat();

    let signature_bytes: &[u8] = binding.as_slice();

    Signature::from_bytes(signature_bytes).map_err(|e| anyhow::anyhow!("Failed to create signature: {}", e))
  }
}

impl<S> HTFClient<S>
where
  S: Signer<IotaKeySignature>,
{
  /// Creates a new federation.
  ///
  /// # Returns
  /// The ID of the newly created federation.
  pub async fn new_federation(&self) -> anyhow::Result<Federation> {
    let federation = federation::ops::create_new_federation(self).await?;

    let federation = self.get_object_by_id(federation).await?;

    Ok(federation)
  }

  /// Adds a root authority to a federation.
  ///
  /// The root authority is an account that has the ability to add other
  /// authorities to the federation.
  pub async fn add_root_authority(&self, federation_id: ObjectID, account_id: ID) -> anyhow::Result<()> {
    federation::ops::add_root_authority(self, federation_id, account_id).await
  }

  /// Adds a trusted property to a federation.
  pub async fn add_trusted_property(
    &self,
    federation_id: ObjectID,
    property_name: TrustedPropertyName,
    allowed_values: HashSet<TrustedPropertyValue>,
    allow_any: bool,
  ) -> anyhow::Result<()> {
    federation::ops::add_trusted_property(self, federation_id, property_name, allowed_values, allow_any).await
  }

  /// Issues a credential for an account in a federation.
  ///
  /// # Arguments
  /// * `federation_id` - The ID of the federation.
  /// * `account_id` - The ID of the account to issue the credential for.
  /// * `trusted_properties` - A map of trusted property names to their values.
  /// * `valid_from_ts` - The timestamp from which the credential is valid.
  /// * `valid_until_ts` - The timestamp until which the credential is valid.
  ///
  /// # Returns
  /// A Result indicating success or failure.
  pub async fn issue_credential(
    &self,
    federation_id: ObjectID,
    account_id: ID,
    trusted_properties: HashMap<TrustedPropertyName, TrustedPropertyValue>,
    valid_from_ts: u64,
    valid_until_ts: u64,
  ) -> anyhow::Result<()> {
    federation::ops::issue_credential(
      self,
      federation_id,
      account_id,
      trusted_properties,
      valid_from_ts,
      valid_until_ts,
    )
    .await
  }

  /// Revokes a permission to attest for a user in a federation.
  ///
  /// # Arguments
  /// * `federation_id` - The ID of the federation.
  /// * `user_id` - The ID of the user whose permission is being revoked.
  /// * `permission_id` - The ID of the permission being revoked.
  ///
  /// # Returns
  /// A Result indicating success or failure.
  pub async fn revoke_permission_to_attest(
    &self,
    federation_id: ObjectID,
    user_id: ID,
    permission_id: ID,
  ) -> anyhow::Result<()> {
    federation::ops::revoke_permission_to_attest(self, federation_id, user_id, permission_id).await
  }

  /// Issues a permission to accredit to a receiver in a federation.
  ///
  /// # Arguments
  /// * `federation_id` - The ID of the federation.
  /// * `receiver` - The ID of the receiver of the permission.
  /// * `want_property_constraints` - A vector of trusted property constraints.
  ///
  /// # Returns
  /// A Result indicating success or failure.
  pub async fn issue_permission_to_accredit(
    &self,
    federation_id: ObjectID,
    receiver: ID,
    want_property_constraints: Vec<TrustedPropertyConstraints>,
  ) -> anyhow::Result<()> {
    federation::ops::issue_permission_to_accredit(self, federation_id, receiver, want_property_constraints).await
  }

  /// Validates a credential in a federation.
  ///
  /// # Arguments
  /// * `federation_id` - The ID of the federation.
  /// * `credential_id` - The ID of the credential to validate.
  ///
  /// # Returns
  /// A Result indicating success or failure.
  pub async fn validate_credential(&self, federation_id: ObjectID, credential_id: ObjectID) -> anyhow::Result<()> {
    federation::ops::validate_credential(self, federation_id, credential_id).await
  }

  /// Issues a permission to attest to a receiver in a federation.
  ///
  /// # Arguments
  /// * `federation_id` - The ID of the federation.
  /// * `receiver` - The ID of the receiver of the permission.
  /// * `want_property_constraints` - A vector of trusted property constraints.
  ///
  /// # Returns
  /// A Result indicating success or failure.
  pub async fn issue_permission_to_attest(
    &self,
    federation_id: ObjectID,
    receiver: ID,
    want_property_constraints: Vec<TrustedPropertyConstraints>,
  ) -> anyhow::Result<()> {
    federation::ops::issue_permission_to_attest(self, federation_id, receiver, want_property_constraints).await
  }

  /// Revokes a permission to accredit for a user in a federation.
  ///
  /// # Arguments
  /// * `federation_id` - The ID of the federation.
  /// * `user_id` - The ID of the user whose permission is being revoked.
  /// * `permission_id` - The ID of the permission being revoked.
  ///
  /// # Returns
  /// A Result indicating success or failure.
  pub async fn revoke_permission_to_accredit(
    &self,
    federation_id: ObjectID,
    user_id: ID,
    permission_id: ID,
  ) -> anyhow::Result<()> {
    federation::ops::revoke_permission_to_accredit(self, federation_id, user_id, permission_id).await
  }
}
impl<S> Deref for HTFClient<S> {
  type Target = HTFClientReadOnly;

  fn deref(&self) -> &Self::Target {
    &self.read_client
  }
}
