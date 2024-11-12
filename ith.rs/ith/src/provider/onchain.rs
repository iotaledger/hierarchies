use std::collections::HashMap;
use std::str::FromStr;

use iota_sdk::types::base_types::{IotaAddress, ObjectID};
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::{Argument, CallArg, Command, ObjectArg, TransactionKind};
use iota_sdk::types::{Identifier, TypeTag};
use move_core_types::ident_str;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::client::ITHClientReadOnly;
use crate::types::Permissions;
use crate::types::{TrustedPropertyName, TrustedPropertyValue};

pub struct OnChainFederation<'c> {
  federation_id: ObjectID,
  client: &'c ITHClientReadOnly,
}

impl<'c> OnChainFederation<'c> {
  pub fn new(client: &'c ITHClientReadOnly, federation_id: ObjectID) -> Self {
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
      self.client.ith_package_id(),
      Identifier::from_str("main")?,
      Identifier::from_str(function_name)?,
      vec![],
      vec![fed_ref, arg],
    )?;

    let tx = TransactionKind::programmable(ptb.finish());

    let sender = IotaAddress::ZERO;

    let result = self
      .client
      .read_api()
      .dev_inspect_transaction_block(sender, tx, None, None, None)
      .await?
      .results
      .and_then(|res| res.first().cloned())
      .ok_or_else(|| anyhow::anyhow!("no results found"))?;

    let (return_value, _) = result
      .return_values
      .first()
      .ok_or_else(|| anyhow::anyhow!("no return values"))?;

    let res: R = bcs::from_bytes(return_value)
      .map_err(|e| anyhow::anyhow!("Failed to deserialize result: {}", e))?;

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
  pub async fn is_accreditor(&self, user_id: ObjectID) -> anyhow::Result<bool> {
    self.execute_query("is_accreditor", user_id).await
  }
  pub async fn is_trusted_property(
    &self,
    property_name: &TrustedPropertyName,
  ) -> anyhow::Result<bool> {
    self
      .execute_query("is_trusted_property", property_name)
      .await
  }

  pub async fn validate_trusted_properties(
    &self,
    issuer_id: ObjectID,
    trusted_properties: impl IntoIterator<Item = (TrustedPropertyName, TrustedPropertyValue)>,
  ) -> anyhow::Result<()> {
    let trusted_properties: HashMap<_, _> = trusted_properties.into_iter().collect();
    let mut ptb = ProgrammableTransactionBuilder::new();

    let fed_ref = ObjectArg::SharedObject {
      id: self.federation_id,
      initial_shared_version: self
        .client
        .initial_shared_version(&self.federation_id)
        .await?,
      mutable: false,
    };

    let fed_ref = ptb.obj(fed_ref)?;

    let mut property_names: Vec<_> = vec![];
    let mut property_values: Vec<_> = vec![];

    for (property_name, property_value) in trusted_properties {
      let names = property_name.names();
      let name = ptb.pure(names)?;
      let property_name: Argument = ptb.programmable_move_call(
        self.client.ith_package_id(),
        ident_str!("trusted_property").into(),
        ident_str!("new_property_name_from_vector").into(),
        vec![],
        vec![name],
      );
      property_names.push(property_name);

      let property_value = match property_value {
        TrustedPropertyValue::Text(text) => {
          let v = ptb.pure(text)?;
          ptb.programmable_move_call(
            self.client.ith_package_id(),
            ident_str!("trusted_property").into(),
            ident_str!("new_property_value_string").into(),
            vec![],
            vec![v],
          )
        }
        TrustedPropertyValue::Number(number) => {
          let v = ptb.pure(number)?;
          ptb.programmable_move_call(
            self.client.ith_package_id(),
            ident_str!("trusted_property").into(),
            ident_str!("new_property_value_number").into(),
            vec![],
            vec![v],
          )
        }
      };
      property_values.push(property_value);
    }

    let property_name_tag = TypeTag::from_str(
      format!(
        "{}::trusted_property::TrustedPropertyName",
        self.client.ith_package_id()
      )
      .as_str(),
    )?;
    let property_value_tag = TypeTag::from_str(
      format!(
        "{}::trusted_property::TrustedPropertyValue",
        self.client.ith_package_id()
      )
      .as_str(),
    )?;

    let property_names = ptb.command(Command::MakeMoveVec(
      Some(property_name_tag.clone()),
      property_names,
    ));
    let property_values = ptb.command(Command::MakeMoveVec(
      Some(property_value_tag.clone()),
      property_values,
    ));

    let trusted_properties = ptb.programmable_move_call(
      self.client.ith_package_id(),
      ident_str!("utils").into(),
      ident_str!("vec_map_from_keys_values").into(),
      vec![property_name_tag, property_value_tag],
      vec![property_names, property_values],
    );

    let issuer_id = ptb.pure(issuer_id)?;

    ptb.programmable_move_call(
      self.client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("validate_trusted_properties").into(),
      vec![],
      vec![fed_ref, issuer_id, trusted_properties],
    );

    let tx = TransactionKind::programmable(ptb.finish());

    let sender = IotaAddress::ZERO;

    let result = self
      .client
      .read_api()
      .dev_inspect_transaction_block(sender, tx, None, None, None)
      .await?;

    if result.error.is_some() {
      anyhow::bail!("Transaction failed: {}", result.error.unwrap());
    }

    Ok(())
  }

  pub async fn get_trusted_properties(&self) -> anyhow::Result<Vec<TrustedPropertyName>> {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let fed_ref = ObjectArg::SharedObject {
      id: self.federation_id,
      initial_shared_version: self
        .client
        .initial_shared_version(&self.federation_id)
        .await?,
      mutable: false,
    };

    let fed_ref = ptb.obj(fed_ref)?;

    ptb.programmable_move_call(
      self.client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("get_trusted_properties").into(),
      vec![],
      vec![fed_ref],
    );

    let tx = TransactionKind::programmable(ptb.finish());

    let sender = IotaAddress::ZERO;

    let result = self
      .client
      .read_api()
      .dev_inspect_transaction_block(sender, tx, None, None, None)
      .await?
      .results
      .and_then(|res| res.first().cloned())
      .ok_or_else(|| anyhow::anyhow!("no results found"))?;

    let (return_value, _) = result
      .return_values
      .first()
      .ok_or_else(|| anyhow::anyhow!("no return values"))?;

    let res: Vec<TrustedPropertyName> = bcs::from_bytes(return_value)
      .map_err(|e| anyhow::anyhow!("Failed to deserialize result: {}", e))?;

    Ok(res)
  }

  pub async fn get_attestations(&self, user_id: ObjectID) -> anyhow::Result<Permissions> {
    self.execute_query("get_attestations", user_id).await
  }

  pub async fn get_accreditations(&self, user_id: ObjectID) -> anyhow::Result<Permissions> {
    self.execute_query("get_accreditations", user_id).await
  }
}
