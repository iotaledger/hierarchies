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
use crate::types::Accreditations;
use crate::types::{StatementName, StatementValue};

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
  pub async fn has_accreditation_to_attest(&self, user_id: ObjectID) -> anyhow::Result<bool> {
    self
      .execute_query("has_accreditation_to_attest", user_id)
      .await
  }
  pub async fn is_accreditor(&self, user_id: ObjectID) -> anyhow::Result<bool> {
    self.execute_query("is_accreditor", user_id).await
  }
  pub async fn is_trustedstatement(&self, statement_name: &StatementName) -> anyhow::Result<bool> {
    self
      .execute_query("is_trustedstatement", statement_name)
      .await
  }

  pub async fn validatestatements(
    &self,
    issuer_id: ObjectID,
    trustedstatements: impl IntoIterator<Item = (StatementName, StatementValue)>,
  ) -> anyhow::Result<()> {
    let trustedstatements: HashMap<_, _> = trustedstatements.into_iter().collect();
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

    let mut statement_names: Vec<_> = vec![];
    let mut property_values: Vec<_> = vec![];

    for (statement_name, property_value) in trustedstatements {
      let names = statement_name.names();
      let name = ptb.pure(names)?;
      let statement_name: Argument = ptb.programmable_move_call(
        self.client.ith_package_id(),
        ident_str!("trustedstatement").into(),
        ident_str!("newstatement_name_from_vector").into(),
        vec![],
        vec![name],
      );
      statement_names.push(statement_name);

      let property_value = match property_value {
        StatementValue::Text(text) => {
          let v = ptb.pure(text)?;
          ptb.programmable_move_call(
            self.client.ith_package_id(),
            ident_str!("trustedstatement").into(),
            ident_str!("new_property_value_string").into(),
            vec![],
            vec![v],
          )
        }
        StatementValue::Number(number) => {
          let v = ptb.pure(number)?;
          ptb.programmable_move_call(
            self.client.ith_package_id(),
            ident_str!("trustedstatement").into(),
            ident_str!("new_property_value_number").into(),
            vec![],
            vec![v],
          )
        }
      };
      property_values.push(property_value);
    }

    let statement_name_tag = TypeTag::from_str(
      format!(
        "{}::trustedstatement::StatementName",
        self.client.ith_package_id()
      )
      .as_str(),
    )?;
    let property_value_tag = TypeTag::from_str(
      format!(
        "{}::trustedstatement::StatementValue",
        self.client.ith_package_id()
      )
      .as_str(),
    )?;

    let statement_names = ptb.command(Command::MakeMoveVec(
      Some(statement_name_tag.clone()),
      statement_names,
    ));
    let property_values = ptb.command(Command::MakeMoveVec(
      Some(property_value_tag.clone()),
      property_values,
    ));

    let trustedstatements = ptb.programmable_move_call(
      self.client.ith_package_id(),
      ident_str!("utils").into(),
      ident_str!("vec_map_from_keys_values").into(),
      vec![statement_name_tag, property_value_tag],
      vec![statement_names, property_values],
    );

    let issuer_id = ptb.pure(issuer_id)?;

    ptb.programmable_move_call(
      self.client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("validatestatements").into(),
      vec![],
      vec![fed_ref, issuer_id, trustedstatements],
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

  pub async fn get_trustedstatements(&self) -> anyhow::Result<Vec<StatementName>> {
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
      ident_str!("get_trustedstatements").into(),
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

    let res: Vec<StatementName> = bcs::from_bytes(return_value)
      .map_err(|e| anyhow::anyhow!("Failed to deserialize result: {}", e))?;

    Ok(res)
  }

  pub async fn get_accreditations_to_attest(
    &self,
    user_id: ObjectID,
  ) -> anyhow::Result<Accreditations> {
    self
      .execute_query("get_accreditations_to_attest", user_id)
      .await
  }

  pub async fn get_accreditations_to_accredit(
    &self,
    user_id: ObjectID,
  ) -> anyhow::Result<Accreditations> {
    self
      .execute_query("get_accreditations_to_accredit", user_id)
      .await
  }
}
