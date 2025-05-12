use std::collections::HashSet;
use std::str::FromStr;

use iota_sdk::rpc_types::{IotaObjectDataFilter, IotaObjectResponseQuery};
use iota_sdk::types::base_types::{IotaAddress, ObjectID, ObjectRef};
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::ObjectArg;
use move_core_types::ident_str;
use move_core_types::language_storage::StructTag;
use secret_storage::Signer;

use crate::client::ITHClient;
use crate::key::IotaKeySignature;
use crate::types::{Event, FederationCreatedEvent};
use crate::types::{StatementName, StatementValue};

use iota_sdk::types::transaction::Argument;

use crate::types::{self, Statement};
use crate::utils::{self, MoveType};

pub(crate) mod ops {
  use types::{
    new_property_constraint, new_property_value_number, new_property_value_string,
    newstatement_name,
  };

  use crate::types::Capability;

  use super::*;

  pub async fn create_new_federation<S>(
    client: &ITHClient<S>,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<ObjectID>
  where
    S: Signer<IotaKeySignature>,
  {
    let mut ptb = ProgrammableTransactionBuilder::new();

    ptb.move_call(
      client.ith_package_id(),
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
      .map(|data| bcs::from_bytes(data.bcs.bytes()))
      .transpose()?
      .ok_or_else(|| anyhow::anyhow!("missing federation event"))?;

    let fed_address = IotaAddress::from_str(&fed_event.data.federation_address.to_string())?;

    Ok(ObjectID::from(fed_address))
  }

  pub async fn add_trustedstatement<S>(
    client: &ITHClient<S>,
    federation_id: ObjectID,
    statement_name: StatementName,
    allowed_values: HashSet<StatementValue>,
    allow_any: bool,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = get_cap(client, "main", Capability::RootAuthority, None).await?;

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let allow_any = ptb.pure(allow_any)?;

    let statement_names = newstatement_name(statement_name, &mut ptb, client.ith_package_id())?;

    let value_tag = StatementValue::move_type(client.ith_package_id());

    let mut values_of_property = vec![];
    for property_value in allowed_values {
      let value = match property_value {
        StatementValue::Text(text) => {
          new_property_value_string(text, &mut ptb, client.ith_package_id())?
        }
        StatementValue::Number(number) => {
          new_property_value_number(number, &mut ptb, client.ith_package_id())?
        }
      };

      values_of_property.push(value);
    }

    let tpv_vec_set = utils::create_vec_set_from_move_values(
      values_of_property,
      value_tag,
      &mut ptb,
      client.ith_package_id(),
    );

    ptb.programmable_move_call(
      client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("add_trustedstatement").into(),
      vec![],
      vec![fed_ref, cap, statement_names, tpv_vec_set, allow_any],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }

  pub async fn remove_trustedstatement<S>(
    client: &ITHClient<S>,
    federation_id: ObjectID,
    statement_name: StatementName,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = get_cap(client, "main", Capability::RootAuthority, None).await?;

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let statement_name = newstatement_name(statement_name, &mut ptb, client.ith_package_id())?;

    ptb.programmable_move_call(
      client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("remove_trustedstatement").into(),
      vec![],
      vec![fed_ref, cap, statement_name],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }
  pub async fn revoke_accreditation_to_attest<S>(
    client: &ITHClient<S>,
    federation_id: ObjectID,
    user_id: ObjectID,
    permission_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", Capability::Attest, None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let user_id_arg = ptb.pure(user_id)?;
    let permission_id = ptb.pure(permission_id)?;

    ptb.programmable_move_call(
      client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("revoke_accreditation_to_attest").into(),
      vec![],
      vec![fed_ref, cap, user_id_arg, permission_id],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }

  pub async fn add_root_authority<S>(
    client: &ITHClient<S>,
    federation_id: ObjectID,
    account_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", Capability::RootAuthority, None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let account_id_arg = ptb.pure(account_id)?;

    ptb.programmable_move_call(
      client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("add_root_authority").into(),
      vec![],
      vec![fed_ref, cap, account_id_arg],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    let address: IotaAddress = account_id.into();

    let Ok(_) = get_cap(client, "main", Capability::RootAuthority, Some(address)).await else {
      anyhow::bail!("failed to get new authority");
    };

    Ok(())
  }

  pub async fn create_accreditation<S>(
    client: &ITHClient<S>,
    federation_id: ObjectID,
    receiver: ObjectID,
    want_property_constraints: Vec<Statement>,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", Capability::Accredit, None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let receiver_arg = ptb.pure(receiver)?;

    let want_property_constraints =
      new_property_constraint(client.ith_package_id(), &mut ptb, want_property_constraints)?;

    ptb.programmable_move_call(
      client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("create_accreditation").into(),
      vec![],
      vec![fed_ref, cap, receiver_arg, want_property_constraints],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    let Ok(_) = get_cap(client, "main", Capability::Accredit, Some(receiver.into())).await else {
      anyhow::bail!("failed to get new accredit");
    };

    Ok(())
  }

  pub async fn create_attestation<S>(
    client: &ITHClient<S>,
    federation_id: ObjectID,
    receiver: ObjectID,
    want_property_constraints: Vec<Statement>,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", Capability::Attest, None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let receiver_arg = ptb.pure(receiver)?;

    let property_constraints =
      new_property_constraint(client.ith_package_id(), &mut ptb, want_property_constraints)?;

    ptb.programmable_move_call(
      client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("create_attestation").into(),
      vec![],
      vec![fed_ref, cap, receiver_arg, property_constraints],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    // Check if the ID has AttestCap
    let Ok(_) = get_cap(client, "main", Capability::Attest, Some(receiver.into())).await else {
      anyhow::bail!("failed to get new accredit");
    };

    Ok(())
  }

  pub async fn revoke_accreditation_to_accredit<S>(
    client: &ITHClient<S>,
    federation_id: ObjectID,
    user_id: ObjectID,
    permission_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap = get_cap(client, "main", Capability::Accredit, None).await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let user_id_arg = ptb.pure(user_id)?;
    let permission_id = ptb.pure(permission_id)?;

    ptb.programmable_move_call(
      client.ith_package_id(),
      ident_str!("main").into(),
      ident_str!("revoke_accreditation_to_accredit").into(),
      vec![],
      vec![fed_ref, cap, user_id_arg, permission_id],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }

  /// Helper function to get a capability of an address
  async fn get_cap<S>(
    client: &ITHClient<S>,
    module: &str,
    cap_type: Capability,
    address: Option<IotaAddress>,
  ) -> anyhow::Result<ObjectRef>
  where
    S: Signer<IotaKeySignature>,
  {
    let cap_tag = StructTag::from_str(&format!(
      "{}::{module}::{cap_type}",
      client.ith_package_id()
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

  /// Get the federation reference for the given federation id
  ///
  /// Since the federation is shared, we need to get the reference to it
  /// to be able to pass it to the programmable transaction builder for other
  /// operations
  async fn get_fed_ref<S>(
    client: &ITHClient<S>,
    federation_id: ObjectID,
    ptb: &mut ProgrammableTransactionBuilder,
  ) -> anyhow::Result<Argument>
  where
    S: Signer<IotaKeySignature>,
  {
    ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: true,
    })
  }
}
