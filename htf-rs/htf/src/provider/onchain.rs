use std::str::FromStr;

use axum::async_trait;
use iota_sdk::types::base_types::IotaAddress;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::collection_types::VecMap;
use iota_sdk::types::id::ID;
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::CallArg;
use iota_sdk::types::transaction::ObjectArg;
use iota_sdk::types::transaction::TransactionKind;
use iota_sdk::types::Identifier;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::client::HTFClientReadOnly;
use crate::federation::FederationReadOperations;
use crate::types::trusted_property::TrustedPropertyName;
use crate::types::trusted_property::TrustedPropertyValue;

pub struct OnChainFederation<'c> {
  federation_id: ObjectID,
  client: &'c HTFClientReadOnly,
}

impl<'c> OnChainFederation<'c> {
  pub fn new(client: &'c HTFClientReadOnly, federation_id: ObjectID) -> Self {
    Self { federation_id, client }
  }

  async fn execute_query<T: Serialize, R: DeserializeOwned>(&self, function_name: &str, arg: T) -> anyhow::Result<R> {
    let mut ptb = ProgrammableTransactionBuilder::new();
    let arg = ptb.pure(arg)?;

    let fed_ref = ObjectArg::SharedObject {
      id: self.federation_id,
      initial_shared_version: self.client.initial_shared_version(&self.federation_id).await?,
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

    let return_values = self
      .client
      .read_api()
      .dev_inspect_transaction_block(sender, tx, None, None, None)
      .await?
      .results
      .and_then(|res| res.first().cloned())
      .ok_or_else(|| anyhow::anyhow!("no results"))?
      .return_values;

    let (res_bytes, _) = &return_values[0];
    let res: R = bcs::from_bytes(res_bytes)?;

    Ok(res)
  }
}

#[async_trait]
impl FederationReadOperations for OnChainFederation<'_> {
  async fn federation_id(&self) -> ObjectID {
    self.federation_id
  }
  async fn has_permission_to_attest(&self, user_id: ID) -> anyhow::Result<bool> {
    self.execute_query("has_permission_to_attest", user_id).await
  }
  async fn has_permissions_to_accredit(&self, user_id: ID) -> anyhow::Result<bool> {
    self.execute_query("has_permissions_to_accredit", user_id).await
  }
  async fn has_federation_property(&self, property_name: &TrustedPropertyName) -> anyhow::Result<bool> {
    self.execute_query("has_federation_property", property_name).await
  }

  async fn validate_trusted_properties(
    &self,
    issuer_id: ID,
    trusted_properties: VecMap<TrustedPropertyName, TrustedPropertyValue>,
  ) -> anyhow::Result<()> {
    self
      .execute_query("validate_trusted_properties", (issuer_id, trusted_properties))
      .await
  }

  async fn get_federation_properties(&self) -> anyhow::Result<Vec<TrustedPropertyName>> {
    self.execute_query("get_federation_properties", ()).await
  }
}
