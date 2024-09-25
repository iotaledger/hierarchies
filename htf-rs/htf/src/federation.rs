use std::collections::HashSet;
use std::str::FromStr;

use iota_sdk::rpc_types::{IotaObjectDataFilter, IotaObjectResponseQuery};
use iota_sdk::types::base_types::{IotaAddress, ObjectID, ObjectRef};
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::ObjectArg;
use move_core_types::ident_str;
use move_core_types::language_storage::StructTag;
use secret_storage::Signer;

use crate::client::HTFClient;
use crate::key::IotaKeySignature;
use crate::types::event::{Event, FederationCreatedEvent};
use crate::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};

use iota_sdk::types::transaction::Argument;

use crate::types::trusted_constraints::{self, TrustedPropertyConstraint};
use crate::types::trusted_property;
use crate::utils::{self, MoveType};

pub(crate) mod ops {
  use crate::types::cap::Capability;

  use super::*;

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

    let cap = get_cap(
      client,
      "main",
      Capability::RootAuthority,
      Some(client.sender_address()),
    )
    .await?;

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;

    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let allow_any = ptb.pure(allow_any)?;

    let property_names =
      trusted_property::new_property_name(property_name, &mut ptb, client.htf_package_id())?;

    let value_tag = TrustedPropertyValue::move_type(client.htf_package_id());

    let mut values_of_property = vec![];
    for property_value in allowed_values {
      let value = match property_value {
        TrustedPropertyValue::Text(text) => {
          trusted_property::new_property_value_string(text, &mut ptb, client.htf_package_id())?
        }
        TrustedPropertyValue::Number(number) => {
          trusted_property::new_property_value_number(number, &mut ptb, client.htf_package_id())?
        }
      };

      values_of_property.push(value);
    }

    let tpv_vec_set = utils::create_vec_set_from_move_values(
      values_of_property,
      value_tag,
      &mut ptb,
      client.htf_package_id(),
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

  pub async fn remove_trusted_property<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    property_name: TrustedPropertyName,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = get_cap(
      client,
      "main",
      Capability::RootAuthority,
      Some(client.sender_address()),
    )
    .await?;

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

    let property_name =
      trusted_property::new_property_name(property_name, &mut ptb, client.htf_package_id())?;

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("remove_trusted_property").into(),
      vec![],
      vec![fed_ref, cap, property_name],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }
  pub async fn revoke_attestation<S>(
    client: &HTFClient<S>,
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
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("revoke_attestation").into(),
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
    let cap = get_cap(
      client,
      "main",
      Capability::RootAuthority,
      Some(client.sender_address()),
    )
    .await?;

    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = get_fed_ref(client, federation_id, &mut ptb).await?;

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

    let Ok(_) = get_cap(client, "main", Capability::RootAuthority, Some(address)).await else {
      anyhow::bail!("failed to get new authority");
    };

    Ok(())
  }

  pub async fn create_accreditation<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    receiver: ObjectID,
    want_property_constraints: Vec<TrustedPropertyConstraint>,
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

    let want_property_constraints = trusted_constraints::create_property_constraints(
      client.htf_package_id(),
      &mut ptb,
      want_property_constraints,
    )?;

    ptb.programmable_move_call(
      client.htf_package_id(),
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
    client: &HTFClient<S>,
    federation_id: ObjectID,
    receiver: ObjectID,
    want_property_constraints: Vec<TrustedPropertyConstraint>,
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

    let property_constraints = trusted_constraints::create_property_constraints(
      client.htf_package_id(),
      &mut ptb,
      want_property_constraints,
    )?;

    ptb.programmable_move_call(
      client.htf_package_id(),
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

  pub async fn revoke_accreditation<S>(
    client: &HTFClient<S>,
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
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("revoke_accreditation").into(),
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
    cap_type: Capability,
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

  /// Get the federation reference for the given federation id
  ///
  /// Since the federation is shared, we need to get the reference to it
  /// to be able to pass it to the programmable transaction builder for other
  /// operations
  async fn get_fed_ref<S>(
    client: &HTFClient<S>,
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
