use fastcrypto::hash::HashFunction;
use fastcrypto::traits::ToFromBytes;

use iota_sdk::rpc_types::IotaExecutionStatus;
use iota_sdk::rpc_types::IotaTransactionBlockEffects;
use iota_sdk::rpc_types::IotaTransactionBlockEffectsV1;
use iota_sdk::rpc_types::IotaTransactionBlockResponse;
use iota_sdk::rpc_types::IotaTransactionBlockResponseOptions;
use iota_sdk::types::base_types::IotaAddress;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::collection_types::VecMap;
use iota_sdk::types::collection_types::VecSet;
use iota_sdk::types::crypto::DefaultHash;
use iota_sdk::types::crypto::Signature;
use iota_sdk::types::crypto::SignatureScheme;
use iota_sdk::types::id::ID;
use iota_sdk::types::quorum_driver_types::ExecuteTransactionRequestType;
use iota_sdk::types::transaction::ProgrammableTransaction;
use iota_sdk::types::transaction::Transaction;
use iota_sdk::types::transaction::TransactionData;

use secret_storage::Signer;
use shared_crypto::intent::Intent;
use shared_crypto::intent::IntentMessage;
use std::ops::Deref;

use crate::federation;
use crate::key::IotaKeySignature;
use crate::key::SigningInfo;

use crate::types::trusted_constraints::TrustedPropertyConstraints;
use crate::types::trusted_property::TrustedPropertyName;
use crate::types::trusted_property::TrustedPropertyValue;
use crate::utils::convert_to_address;

use super::HTFClientReadOnly;

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
  /// Creates a new `HTFClient` instance with the given IOTA network URL and
  /// the HTF package ID.
  ///
  /// This function initializes an `IotaClient` instance using the provided
  /// URL and returns a new `HTFClient`
  /// instance with the given HTF package ID.
  ///
  /// # Arguments
  /// * `url` - The URL of the IOTA network to connect to.
  /// * `package_id` - The package ID of the HTF package.
  ///
  /// # Returns
  /// A new `HTFClient` instance, or an error if the `IotaClient` could not
  /// be created.
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
  pub async fn new_federation(&self) -> anyhow::Result<ObjectID> {
    let federation = federation::ops::create_new_federation(self).await?;

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
    allowed_values: VecSet<TrustedPropertyValue>,
    allow_any: bool,
  ) -> anyhow::Result<()> {
    federation::ops::add_trusted_property(self, federation_id, property_name, allowed_values, allow_any).await
  }

  pub async fn issue_credential(
    &self,
    federation_id: ObjectID,
    account_id: ID,
    trusted_properties: VecMap<TrustedPropertyName, TrustedPropertyValue>,
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

  pub async fn revoke_permission_to_attest(
    &self,
    federation_id: ObjectID,
    user_id: ID,
    permission_id: ID,
  ) -> anyhow::Result<()> {
    federation::ops::revoke_permission_to_attest(self, federation_id, user_id, permission_id).await
  }

  pub async fn issue_permission_to_accredit(
    &self,
    federation_id: ObjectID,
    receiver: ID,
    want_property_constraints: Vec<TrustedPropertyConstraints>,
  ) -> anyhow::Result<()> {
    federation::ops::issue_permission_to_accredit(self, federation_id, receiver, want_property_constraints).await
  }

  pub async fn validate_credential(&self, federation_id: ObjectID, credential_id: ObjectID) -> anyhow::Result<()> {
    federation::ops::validate_credential(self, federation_id, credential_id).await
  }

  pub async fn issue_permission_to_attest(
    &self,
    federation_id: ObjectID,
    receiver: ID,
    want_property_constraints: Vec<TrustedPropertyConstraints>,
  ) -> anyhow::Result<()> {
    federation::ops::issue_permission_to_attest(self, federation_id, receiver, want_property_constraints).await
  }

  /// Revokes a permission to accredit.
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
