use std::str::FromStr;

use iota_sdk::types::base_types::{IotaAddress, ObjectID};
use iota_sdk::types::collection_types::VecMap;
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::{CallArg, ObjectArg, TransactionKind};
use iota_sdk::types::Identifier;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::client::HTFClientReadOnly;
use crate::types::permission::{PermissionsToAccredit, PermissionsToAttest};
use crate::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};

pub struct OnChainFederation<'c> {
  federation_id: ObjectID,
  client: &'c HTFClientReadOnly,
}

impl<'c> OnChainFederation<'c> {
  pub fn new(client: &'c HTFClientReadOnly, federation_id: ObjectID) -> Self {
    Self {
      federation_id,
      client,
    }
  }

  async fn execute_query<T: Serialize, R: DeserializeOwned>(
    &self,
    function_name: &str,
    arg: T,
  ) -> anyhow::Result<R> {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let fed_ref = ObjectArg::SharedObject {
      id: self.federation_id,
      initial_shared_version: self
        .client
        .initial_shared_version(&self.federation_id)
        .await?,
      mutable: false,
    };

    let arg = CallArg::Pure(bcs::to_bytes(&arg)?);
    let fed_ref = CallArg::Object(fed_ref);

    ptb.move_call(
      self.client.htf_package_id(),
      Identifier::from_str("main")?,
      Identifier::from_str(function_name)?,
      vec![],
      vec![fed_ref, arg],
    )?;

    let tx = TransactionKind::programmable(ptb.finish());

    let sender = IotaAddress::ZERO; //TODO::fix this

    let result = self
      .client
      .read_api()
      .dev_inspect_transaction_block(sender, tx, None, None, None)
      .await?
      .results
      .and_then(|res| res.first().cloned())
      .ok_or_else(|| anyhow::anyhow!("no results"))?;

    println!("result: {:?}", result);

    let (return_value, _) = result
      .return_values
      .first()
      .ok_or_else(|| anyhow::anyhow!("no return values"))?;

    println!("return value: {:?}", return_value);

    let res: R = bcs::from_bytes(return_value).map_err(|e| anyhow::anyhow!("Failed to deserialize result: {}", e))?;

    Ok(res)
  }
}

impl OnChainFederation<'_> {
  pub async fn federation_id(&self) -> ObjectID {
    self.federation_id
  }
  pub async fn has_permission_to_attest(&self, user_id: ObjectID) -> anyhow::Result<bool> {
    self
      .execute_query("has_permission_to_attest", user_id)
      .await
  }
  pub async fn has_permissions_to_accredit(&self, user_id: ObjectID) -> anyhow::Result<bool> {
    self
      .execute_query("has_permissions_to_accredit", user_id)
      .await
  }
  pub async fn has_federation_property(
    &self,
    property_name: &TrustedPropertyName,
  ) -> anyhow::Result<bool> {
    self
      .execute_query("has_federation_property", property_name)
      .await
  }

  pub async fn validate_trusted_properties(
    &self,
    issuer_id: ObjectID,
    trusted_properties: VecMap<TrustedPropertyName, TrustedPropertyValue>,
  ) -> anyhow::Result<()> {
    self
      .execute_query(
        "validate_trusted_properties",
        (issuer_id, trusted_properties),
      )
      .await
  }

  pub async fn get_federation_properties(&self) -> anyhow::Result<Vec<TrustedPropertyName>> {
    self.execute_query("get_federation_properties", ()).await
  }

  pub async fn find_permissions_to_attest(&self, user_id: ObjectID) -> anyhow::Result<PermissionsToAttest> {
    self.execute_query("find_permissions_to_attest", user_id).await
  }

  pub async fn find_permissions_to_accredit(&self, user_id: ObjectID) -> anyhow::Result<PermissionsToAccredit> {
    self.execute_query("find_permissions_to_accredit", user_id).await
  }
}
