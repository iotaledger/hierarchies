use std::collections::HashSet;
use std::ops::Deref;

use fastcrypto::hash::HashFunction;
use fastcrypto::traits::ToFromBytes;
use iota_sdk::rpc_types::{
  IotaExecutionStatus, IotaTransactionBlockEffects, IotaTransactionBlockEffectsAPI,
  IotaTransactionBlockEffectsV1, IotaTransactionBlockResponse, IotaTransactionBlockResponseOptions,
};
use iota_sdk::types::base_types::{IotaAddress, ObjectID};
use iota_sdk::types::crypto::{DefaultHash, Signature, SignatureScheme};
use iota_sdk::types::quorum_driver_types::ExecuteTransactionRequestType;
use iota_sdk::types::transaction::{ProgrammableTransaction, Transaction, TransactionData};
use secret_storage::Signer;
use shared_crypto::intent::{Intent, IntentMessage};

use super::ITHClientReadOnly;
use crate::federation;
use crate::key::{IotaKeySignature, SigningInfo};
use crate::types::Federation;
use crate::types::TrustedPropertyConstraint;
use crate::types::{TrustedPropertyName, TrustedPropertyValue};
use crate::utils::convert_to_address;

/// The `ITHClient` struct is responsible for managing the connection to the
/// IOTA network and executing transactions on behalf of the ITH package.
pub struct ITHClient<S> {
  read_client: ITHClientReadOnly,
  signing_info: SigningInfo,
  signer: S,
}

impl<S> ITHClient<S>
where
  S: Signer<IotaKeySignature>,
{
  /// Creates a new [`ITHClient`] instance.
  ///
  /// This function initializes an `ITHClient` with the provided [`ITHClientReadOnly`], `Signer`, and gas budget.
  /// The `SigningInfo` struct is also initialized with the signer's public key and the derived sender address.
  ///
  /// # Arguments
  /// * `read_client` - The [`ITHClientReadOnly`] instance to use for read operations.
  /// * `signer` - The `Signer` instance to use for signing transactions.
  /// * `gas_budget` - The gas budget to use for transactions.
  ///
  /// # Returns
  /// A new `ITHClient` instance.
  pub async fn new(read_client: ITHClientReadOnly, signer: S) -> anyhow::Result<Self> {
    let pub_key = signer.public_key().await?;
    let address = convert_to_address(&pub_key)?;
    let signing_info = SigningInfo {
      sender_address: address,
      sender_public_key: pub_key,
    };

    Ok(Self {
      read_client,
      signer,
      signing_info,
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
    gas_budget: Option<u64>,
  ) -> anyhow::Result<IotaTransactionBlockResponse> {
    let gas = match gas_budget {
      Some(gas) => gas,
      None => self.estimate_gas(&tx).await?,
    };

    let tx_data = self.get_transaction_data(tx, gas).await?;
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
    gas_budget: u64,
  ) -> anyhow::Result<TransactionData> {
    let gas_price = self
      .read_client
      .read_api()
      .get_reference_gas_price()
      .await?;

    let sender = self.sender_address();

    let coin = self.get_coin_for_transaction().await?;
    let tx_data = TransactionData::new_programmable(
      sender,
      vec![coin.object_ref()],
      programmable_transaction,
      gas_budget,
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
    let SigningInfo {
      sender_public_key, ..
    } = &self.signing_info;

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

    Signature::from_bytes(signature_bytes)
      .map_err(|e| anyhow::anyhow!("Failed to create signature: {}", e))
  }

  /// Estimates the gas budget for a transaction.
  ///
  /// This function calculates the gas budget for a transaction by executing a dry run of the transaction
  /// and returning the gas used with a small buffer.
  pub async fn estimate_gas(&self, tx: &ProgrammableTransaction) -> anyhow::Result<u64> {
    let gas_price = self.read_api().get_reference_gas_price().await?;
    let gas_coin = self.get_coin_for_transaction().await?;

    let tx_data = TransactionData::new_programmable(
      self.sender_address(),
      vec![gas_coin.object_ref()],
      tx.clone(),
      5_000_000_000,
      gas_price,
    );

    let dry_run_gas_result = self
      .read_api()
      .dry_run_transaction_block(tx_data)
      .await?
      .effects;
    if dry_run_gas_result.status().is_err() {
      let IotaExecutionStatus::Failure { error } = dry_run_gas_result.into_status() else {
        unreachable!();
      };

      anyhow::bail!("Failed to dry run transaction: {}", error);
    }

    let gas_summary = dry_run_gas_result.gas_cost_summary();
    let overhead = gas_price * 1000;
    let net_used = gas_summary.net_gas_usage();
    let computation = gas_summary.computation_cost;

    let budget = overhead + (net_used.max(0) as u64).max(computation);
    Ok(budget)
  }
}

impl<S> ITHClient<S>
where
  S: Signer<IotaKeySignature>,
{
  /// Creates a new federation.
  pub async fn new_federation(&self, gas_budget: Option<u64>) -> anyhow::Result<Federation> {
    let federation = federation::ops::create_new_federation(self, gas_budget).await?;

    let federation = self.get_object_by_id(federation).await?;

    Ok(federation)
  }

  /// Adds a root authority to a federation.
  /// The root authority is an account that has the ability to add other
  /// authorities to the federation.
  pub async fn add_root_authority(
    &self,
    federation_id: ObjectID,
    account_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()> {
    federation::ops::add_root_authority(self, federation_id, account_id, gas_budget).await
  }

  /// Adds a trusted property to a federation.
  pub async fn add_trusted_property(
    &self,
    federation_id: ObjectID,
    property_name: impl Into<TrustedPropertyName>,
    allowed_values: impl IntoIterator<Item = TrustedPropertyValue>,
    allow_any: bool,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()> {
    let allowed_values = HashSet::from_iter(allowed_values);
    federation::ops::add_trusted_property(
      self,
      federation_id,
      property_name.into(),
      allowed_values,
      allow_any,
      gas_budget,
    )
    .await
  }

  /// Removes a trusted property from a federation.
  pub async fn remove_trusted_property(
    &self,
    federation_id: ObjectID,
    property_name: TrustedPropertyName,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()> {
    federation::ops::remove_trusted_property(self, federation_id, property_name, gas_budget).await
  }

  /// Issues a permission to attest to a receiver in a federation.
  pub async fn create_attestation(
    &self,
    federation_id: ObjectID,
    receiver: ObjectID,
    want_property_constraints: impl IntoIterator<Item = TrustedPropertyConstraint>,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()> {
    let want_property_constraints = want_property_constraints.into_iter().collect();
    federation::ops::create_attestation(
      self,
      federation_id,
      receiver,
      want_property_constraints,
      gas_budget,
    )
    .await
  }

  /// Revokes a permission to attest for a user in a federation.
  pub async fn revoke_attestation(
    &self,
    federation_id: ObjectID,
    user_id: ObjectID,
    permission_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()> {
    federation::ops::revoke_attestation(self, federation_id, user_id, permission_id, gas_budget)
      .await
  }

  /// Issues a permission to accredit to a receiver in a federation.
  pub async fn create_accreditation(
    &self,
    federation_id: ObjectID,
    receiver: ObjectID,
    want_property_constraints: Vec<TrustedPropertyConstraint>,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()> {
    federation::ops::create_accreditation(
      self,
      federation_id,
      receiver,
      want_property_constraints,
      gas_budget,
    )
    .await
  }

  /// Revokes a permission to accredit for a user in a federation.
  pub async fn revoke_accreditation(
    &self,
    federation_id: ObjectID,
    user_id: ObjectID,
    permission_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()> {
    federation::ops::revoke_accreditation(self, federation_id, user_id, permission_id, gas_budget)
      .await
  }
}

impl<S> Deref for ITHClient<S> {
  type Target = ITHClientReadOnly;

  fn deref(&self) -> &Self::Target {
    &self.read_client
  }
}
