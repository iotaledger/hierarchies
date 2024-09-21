use std::collections::HashSet;
use std::str::FromStr;

use iota_sdk::rpc_types::{IotaObjectDataFilter, IotaObjectResponseQuery, IotaTransactionBlockEffectsAPI};
use iota_sdk::types::base_types::{IotaAddress, ObjectID, ObjectRef};
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::ObjectArg;
use move_core_types::ident_str;
use move_core_types::language_storage::StructTag;
use secret_storage::Signer;

use crate::client::HTFClient;
use crate::key::IotaKeySignature;
use crate::types::event::{Event, FederationCreatedEvent};
use crate::types::trusted_constraints::TrustedPropertyConstraints;
use crate::types::trusted_property::{
  TrustedPropertyName, TrustedPropertyValue, TrustedPropertyValueMove,
};

pub(crate) mod ops {
  use iota_sdk::types::base_types::{STD_OPTION_MODULE_NAME, STD_UTF8_MODULE_NAME};
  
  
  use iota_sdk::types::transaction::{Argument, Command};
  use iota_sdk::types::{
    TypeTag, MOVE_STDLIB_PACKAGE_ID,
  };

  use super::*;
  use crate::types::trusted_constraints::TrustedPropertyConstraint;
  use crate::utils;

  pub async fn create_new_federation<S>(
    client: &HTFClient<S>,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<ObjectID>
  where
    S: Signer<IotaKeySignature>,
  {
    let mut ptb = ProgrammableTransactionBuilder::new();

    ptb.move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("new_federation").into(),
      vec![],
      vec![],
    )?;

    let tx = ptb.finish();

    let iota_tx = client.execute_transaction(tx, gas_budget).await?;

    // Check event emitted
    let fed_event: Event<FederationCreatedEvent> = iota_tx
      .events
      .ok_or_else(|| anyhow::anyhow!("missing events"))?
      .data
      .first()
      .map(|data| bcs::from_bytes(data.bcs.as_slice()))
      .transpose()?
      .ok_or_else(|| anyhow::anyhow!("missing federation event"))?;

    let fed_address = IotaAddress::from_str(&fed_event.data.federation_address.to_string())?;

    Ok(ObjectID::from(fed_address))
  }

  pub async fn add_trusted_property<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    property_name: TrustedPropertyName,
    allowed_values: HashSet<TrustedPropertyValue>,
    allow_any: bool,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = get_cap(client, "main", "RootAuthorityCap", Some(client.sender_address())).await?;

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

    let fed_ref = ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: true,
    })?;

    let allow_any = ptb.pure(allow_any)?;
    let names = ptb.pure(property_name.names())?;
    let property_names: Argument = ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("trusted_property").into(),
      ident_str!("new_property_name_from_vector").into(),
      vec![],
      vec![names],
    );

    let tag =
      TypeTag::from_str(format!("{}::trusted_property::TrustedPropertyValue", client.htf_package_id()).as_str())?;

    let mut values_of_property = vec![];
    for property_value in allowed_values {
      let value = match property_value {
        TrustedPropertyValue::Text(text) => {
          let v = ptb.pure(text)?;
          ptb.programmable_move_call(
            client.htf_package_id(),
            ident_str!("trusted_property").into(),
            ident_str!("new_property_value_string").into(),
            vec![],
            vec![v],
          )
        }
        TrustedPropertyValue::Number(number) => {
          let v = ptb.pure(number)?;
          ptb.programmable_move_call(
            client.htf_package_id(),
            ident_str!("trusted_property").into(),
            ident_str!("new_property_value_number").into(),
            vec![],
            vec![v],
          )
        }
      };

      values_of_property.push(value);
    }

    let tpv_move_vec = ptb.command(Command::MakeMoveVec(Some(tag.clone()), values_of_property));

    let tpv_vec_set = ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("utils").into(),
      ident_str!("create_vec_set").into(),
      vec![tag],
      vec![tpv_move_vec],
    );

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("add_trusted_property").into(),
      vec![],
      vec![fed_ref, cap, property_names, tpv_vec_set, allow_any],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }

  pub async fn revoke_permission_to_attest<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    user_id: ObjectID,
    permission_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", "AttestCap", None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: true,
    })?;

    let user_id_arg = ptb.pure(user_id)?;
    let permission_id = ptb.pure(permission_id)?;

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("revoke_permission_to_attest").into(),
      vec![],
      vec![fed_ref, cap, user_id_arg, permission_id],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }

  pub async fn add_root_authority<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    account_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", "RootAuthorityCap", Some(client.sender_address())).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: true,
    })?;

    let account_id_arg = ptb.pure(account_id)?;

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("add_root_authority").into(),
      vec![],
      vec![fed_ref, cap, account_id_arg],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    let address: IotaAddress = account_id.into();

    let Ok(_) = get_cap(client, "main", "RootAuthorityCap", Some(address)).await else {
      anyhow::bail!("failed to get new authority");
    };

    Ok(())
  }

  pub async fn issue_permission_to_accredit<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    receiver: ObjectID,
    want_property_constraints: Vec<TrustedPropertyConstraint>,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", "AccreditCap", None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: true,
    })?;

    let receiver_arg = ptb.pure(receiver)?;

    let want_property_constraints = {
      let mut constraints = vec![];
      for constraint in want_property_constraints {
        let property_value_tag =
          TypeTag::from_str(format!("{}::trusted_property::TrustedPropertyValue", client.htf_package_id()).as_str())?;

        let names = ptb.pure(constraint.property_name.names())?;
        let property_names: Argument = ptb.programmable_move_call(
          client.htf_package_id(),
          ident_str!("trusted_property").into(),
          ident_str!("new_property_name_from_vector").into(),
          vec![],
          vec![names],
        );

        let allow_any = ptb.pure(constraint.allow_any)?;
        let allowed_values = constraint
          .allowed_values
          .iter()
          .map(|value| match value {
            TrustedPropertyValue::Text(text) => {
              let v = ptb.pure(text).expect("");
              ptb.programmable_move_call(
                client.htf_package_id(),
                ident_str!("trusted_property").into(),
                ident_str!("new_property_value_string").into(),
                vec![],
                vec![v],
              )
            }
            TrustedPropertyValue::Number(number) => {
              let v = ptb.pure(number).expect("");
              ptb.programmable_move_call(
                client.htf_package_id(),
                ident_str!("trusted_property").into(),
                ident_str!("new_property_value_number").into(),
                vec![],
                vec![v],
              )
            }
          })
          .collect();
        let allowed_values = ptb.command(Command::MakeMoveVec(Some(property_value_tag.clone()), allowed_values));

        let allowed_values = ptb.programmable_move_call(
          client.htf_package_id(),
          ident_str!("utils").into(),
          ident_str!("create_vec_set").into(),
          vec![property_value_tag],
          vec![allowed_values],
        );

        let property_expression_tag = TypeTag::from_str(
          format!(
            "{}::trusted_constraint::TrustedPropertyExpression",
            client.htf_package_id()
          )
          .as_str(),
        )?;

        let expression = match constraint.expression {
          Some(expression) => {
            let string_tag = TypeTag::from_str(format!("{}::string::String", MOVE_STDLIB_PACKAGE_ID).as_str())?;

            let starts_with = match expression.as_starts_with() {
              Some(value) => {
                let v = ptb.pure(value.as_bytes())?;
                ptb.programmable_move_call(
                  MOVE_STDLIB_PACKAGE_ID,
                  STD_UTF8_MODULE_NAME.into(),
                  ident_str!("utf8").into(),
                  vec![],
                  vec![v],
                )
              }
              None => utils::option_to_move::<String>(None, string_tag.clone(), &mut ptb)?,
            };

            let ends_with = match expression.as_ends_with() {
              Some(value) => {
                let v = ptb.pure(value.as_bytes())?;
                ptb.programmable_move_call(
                  MOVE_STDLIB_PACKAGE_ID,
                  STD_UTF8_MODULE_NAME.into(),
                  ident_str!("utf8").into(),
                  vec![],
                  vec![v],
                )
              }
              None => utils::option_to_move::<String>(None, string_tag.clone(), &mut ptb)?,
            };

            let contains = match expression.as_contains() {
              Some(value) => {
                let v = ptb.pure(value.as_bytes())?;
                ptb.programmable_move_call(
                  MOVE_STDLIB_PACKAGE_ID,
                  STD_UTF8_MODULE_NAME.into(),
                  ident_str!("utf8").into(),
                  vec![],
                  vec![v],
                )
              }
              None => utils::option_to_move::<String>(None, string_tag.clone(), &mut ptb)?,
            };

            let greater_than = utils::option_to_move(expression.as_greater_than(), TypeTag::U64, &mut ptb)?;
            let lower_than = utils::option_to_move(expression.as_lower_than(), TypeTag::U64, &mut ptb)?;

            let arg = ptb.programmable_move_call(
              client.htf_package_id(),
              ident_str!("trusted_constraint").into(),
              ident_str!("new_trusted_property_expression").into(),
              vec![],
              vec![starts_with, ends_with, contains, greater_than, lower_than],
            );

            ptb.programmable_move_call(
              MOVE_STDLIB_PACKAGE_ID,
              STD_OPTION_MODULE_NAME.into(),
              ident_str!("some").into(),
              vec![property_expression_tag],
              vec![arg],
            )
          }

          None => utils::option_to_move::<TrustedPropertyConstraint>(None, property_expression_tag, &mut ptb)?,
        };

        let constraint = ptb.programmable_move_call(
          client.htf_package_id(),
          ident_str!("trusted_constraint").into(),
          ident_str!("new_trusted_property_constraint").into(),
          vec![],
          vec![property_names, allowed_values, allow_any, expression],
        );
        constraints.push(constraint);
      }

      ptb.command(Command::MakeMoveVec(
        Some(TypeTag::from_str(
          format!(
            "{}::trusted_constraint::TrustedPropertyConstraint",
            client.htf_package_id()
          )
          .as_str(),
        )?),
        constraints,
      ))
    };

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("issue_permission_to_accredit").into(),
      vec![],
      vec![fed_ref, cap, receiver_arg, want_property_constraints],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    let Ok(_) = get_cap(client, "main", "AccreditCap", Some(receiver.into())).await else {
      anyhow::bail!("failed to get new accredit");
    };

    Ok(())
  }

  pub async fn issue_permission_to_attest<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    receiver: ObjectID,
    want_property_constraints: Vec<TrustedPropertyConstraint>,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", "AttestCap", None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: true,
    })?;

    let receiver_arg = ptb.pure(receiver)?;

    let want_property_constraints = {
      let mut constraints = vec![];
      for constraint in want_property_constraints {
        let property_value_tag =
          TypeTag::from_str(format!("{}::trusted_property::TrustedPropertyValue", client.htf_package_id()).as_str())?;

        let names = ptb.pure(constraint.property_name.names())?;
        let property_names: Argument = ptb.programmable_move_call(
          client.htf_package_id(),
          ident_str!("trusted_property").into(),
          ident_str!("new_property_name_from_vector").into(),
          vec![],
          vec![names],
        );

        let allow_any = ptb.pure(constraint.allow_any)?;
        let allowed_values = constraint
          .allowed_values
          .iter()
          .map(|value| match value {
            TrustedPropertyValue::Text(text) => {
              let v = ptb.pure(text).expect("");
              ptb.programmable_move_call(
                client.htf_package_id(),
                ident_str!("trusted_property").into(),
                ident_str!("new_property_value_string").into(),
                vec![],
                vec![v],
              )
            }
            TrustedPropertyValue::Number(number) => {
              let v = ptb.pure(number).expect("");
              ptb.programmable_move_call(
                client.htf_package_id(),
                ident_str!("trusted_property").into(),
                ident_str!("new_property_value_number").into(),
                vec![],
                vec![v],
              )
            }
          })
          .collect();
        let allowed_values = ptb.command(Command::MakeMoveVec(Some(property_value_tag.clone()), allowed_values));

        let allowed_values = ptb.programmable_move_call(
          client.htf_package_id(),
          ident_str!("utils").into(),
          ident_str!("create_vec_set").into(),
          vec![property_value_tag],
          vec![allowed_values],
        );

        let property_expression_tag = TypeTag::from_str(
          format!(
            "{}::trusted_constraint::TrustedPropertyExpression",
            client.htf_package_id()
          )
          .as_str(),
        )?;

        let expression = match constraint.expression {
          Some(expression) => {
            let string_tag = TypeTag::from_str(format!("{}::string::String", MOVE_STDLIB_PACKAGE_ID).as_str())?;

            let starts_with = match expression.as_starts_with() {
              Some(value) => {
                let v = ptb.pure(value.as_bytes())?;
                ptb.programmable_move_call(
                  MOVE_STDLIB_PACKAGE_ID,
                  STD_UTF8_MODULE_NAME.into(),
                  ident_str!("utf8").into(),
                  vec![],
                  vec![v],
                )
              }
              None => utils::option_to_move::<String>(None, string_tag.clone(), &mut ptb)?,
            };

            let ends_with = match expression.as_ends_with() {
              Some(value) => {
                let v = ptb.pure(value.as_bytes())?;
                ptb.programmable_move_call(
                  MOVE_STDLIB_PACKAGE_ID,
                  STD_UTF8_MODULE_NAME.into(),
                  ident_str!("utf8").into(),
                  vec![],
                  vec![v],
                )
              }
              None => utils::option_to_move::<String>(None, string_tag.clone(), &mut ptb)?,
            };

            let contains = match expression.as_contains() {
              Some(value) => {
                let v = ptb.pure(value.as_bytes())?;
                ptb.programmable_move_call(
                  MOVE_STDLIB_PACKAGE_ID,
                  STD_UTF8_MODULE_NAME.into(),
                  ident_str!("utf8").into(),
                  vec![],
                  vec![v],
                )
              }
              None => utils::option_to_move::<String>(None, string_tag.clone(), &mut ptb)?,
            };

            let greater_than = utils::option_to_move(expression.as_greater_than(), TypeTag::U64, &mut ptb)?;
            let lower_than = utils::option_to_move(expression.as_lower_than(), TypeTag::U64, &mut ptb)?;

            let arg = ptb.programmable_move_call(
              client.htf_package_id(),
              ident_str!("trusted_constraint").into(),
              ident_str!("new_trusted_property_expression").into(),
              vec![],
              vec![starts_with, ends_with, contains, greater_than, lower_than],
            );

            ptb.programmable_move_call(
              MOVE_STDLIB_PACKAGE_ID,
              STD_OPTION_MODULE_NAME.into(),
              ident_str!("some").into(),
              vec![property_expression_tag],
              vec![arg],
            )
          }

          None => utils::option_to_move::<TrustedPropertyConstraint>(None, property_expression_tag, &mut ptb)?,
        };

        let constraint = ptb.programmable_move_call(
          client.htf_package_id(),
          ident_str!("trusted_constraint").into(),
          ident_str!("new_trusted_property_constraint").into(),
          vec![],
          vec![property_names, allowed_values, allow_any, expression],
        );
        constraints.push(constraint);
      }

      ptb.command(Command::MakeMoveVec(
        Some(TypeTag::from_str(
          format!(
            "{}::trusted_constraint::TrustedPropertyConstraint",
            client.htf_package_id()
          )
          .as_str(),
        )?),
        constraints,
      ))
    };

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("issue_permission_to_attest").into(),
      vec![],
      vec![fed_ref, cap, receiver_arg, want_property_constraints],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    // Check if the ID has AttestCap
    let Ok(_) = get_cap(client, "main", "AttestCap", Some(receiver.into())).await else {
      anyhow::bail!("failed to get new accredit");
    };

    Ok(())
  }

  pub async fn revoke_permission_to_accredit<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    user_id: ObjectID,
    permission_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", "AccreditCap", None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: true,
    })?;

    let user_id_arg = ptb.pure(user_id)?;
    let permission_id = ptb.pure(permission_id)?;

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("revoke_permission_to_accredit").into(),
      vec![],
      vec![fed_ref, cap, user_id_arg, permission_id],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }

  /// Helper function to get a capability of an address
  async fn get_cap<S>(
    client: &HTFClient<S>,
    module: &str,
    cap_type: &str,
    address: Option<IotaAddress>,
  ) -> anyhow::Result<ObjectRef>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap_tag = StructTag::from_str(&format!(
      "{}::{module}::{cap_type}",
      client.htf_package_id()
    ))?;

    let filter =
      IotaObjectResponseQuery::new_with_filter(IotaObjectDataFilter::StructType(cap_tag));

    let mut cursor = None;
    loop {
      let sender = address.unwrap_or(client.sender_address());

      let mut page = client
        .read_api()
        .get_owned_objects(sender, Some(filter.clone()), cursor, None)
        .await?;
      let cap = std::mem::take(&mut page.data)
        .into_iter()
        .find_map(|res| res.data.map(|obj| obj.object_ref()));

      cursor = page.next_cursor;
      if let Some(cap) = cap {
        return Ok(cap);
      }
      if !page.has_next_page {
        break;
      }
    }

    anyhow::bail!("no cap of type `{cap_type}`",)
  }
}
