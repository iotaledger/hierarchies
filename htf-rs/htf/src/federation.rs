use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use anyhow::Context;
use iota_sdk::rpc_types::{IotaObjectDataFilter, IotaObjectResponseQuery, IotaTransactionBlockEffectsAPI};
use iota_sdk::types::base_types::{IotaAddress, ObjectID, ObjectRef};
use iota_sdk::types::collection_types::{Entry, VecMap};
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::ObjectArg;
use move_core_types::ident_str;
use move_core_types::language_storage::StructTag;
use secret_storage::Signer;

use crate::client::HTFClient;
use crate::key::IotaKeySignature;
use crate::types::credentials::Credential;
use crate::types::event::{Event, FederationCreatedEvent};
use crate::types::trusted_constraints::TrustedPropertyConstraints;
use crate::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue, TrustedPropertyValueMove};

pub(crate) mod ops {
  use iota_sdk::types::collection_types::VecSet;
  use iota_sdk::types::iota_system_state::IOTA_SYSTEM_MODULE_NAME;
  use iota_sdk::types::transaction::{Argument, CallArg, Command};
  use iota_sdk::types::{TypeTag, IOTA_FRAMEWORK_ADDRESS, IOTA_FRAMEWORK_PACKAGE_ID, IOTA_SYSTEM_PACKAGE_ID};
  use serde::Serialize;

  use crate::types::trusted_constraints::TrustedPropertyConstraint;

  use super::*;

  pub async fn create_new_federation<S>(client: &HTFClient<S>, gas_budget: Option<u64>) -> anyhow::Result<ObjectID>
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

    let values_of_property = allowed_values.iter().collect::<Vec<_>>();

    // TODO::@itsyaasir: Fix this
    let value = match values_of_property[0] {
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

    let vec_set = ptb.programmable_move_call(
      IOTA_FRAMEWORK_PACKAGE_ID,
      ident_str!("vec_set").into(),
      ident_str!("singleton").into(),
      vec![tag],
      vec![value],
    );

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("add_trusted_property").into(),
      vec![],
      vec![fed_ref, cap, property_names, vec_set, allow_any],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    Ok(())
  }

  pub async fn issue_credential<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    receiver: ObjectID,
    trusted_properties: HashMap<TrustedPropertyName, TrustedPropertyValue>,
    valid_from_ts: u64,
    valid_until_ts: u64,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let cap = get_cap(client, "main", "AttestCap", None).await?;

    let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
    let fed_ref = ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: true,
    })?;

    let receiver_arg = ptb.pure(receiver)?;

    let trusted_properties_vec = {
      let trusted_properties_vec = trusted_properties
        .into_iter()
        .map(|(k, v)| {
          let v = TrustedPropertyValueMove::from(v);
          Entry { key: k, value: v }
        })
        .collect();

      VecMap {
        contents: trusted_properties_vec,
      }
    };

    let trusted_properties_arg = ptb.pure(&trusted_properties_vec)?;
    let valid_from_ts_arg = ptb.pure(valid_from_ts)?;
    let valid_until_ts_arg = ptb.pure(valid_until_ts)?;

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("issue_credential").into(),
      vec![],
      vec![
        cap,
        fed_ref,
        receiver_arg,
        trusted_properties_arg,
        valid_from_ts_arg,
        valid_until_ts_arg,
      ],
    );

    let tx = ptb.finish();

    let iota_res = client.execute_transaction(tx, gas_budget).await?;

    let created_object = iota_res
      .effects
      .ok_or_else(|| anyhow::anyhow!("missing effects"))?
      .created()
      .first()
      .ok_or_else(|| anyhow::anyhow!("missing created object"))?
      .object_id();

    let cred: Credential = client.get_object_by_id(created_object).await?;

    assert_eq!(cred.issued_for, receiver, "invalid issued_for");

    assert_eq!(cred.valid_from, valid_from_ts, "invalid valid_from");
    assert_eq!(cred.valid_to, valid_until_ts, "invalid valid_until");

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
      ident_str!("revoke_permission_to_accredit").into(),
      vec![],
      vec![cap, fed_ref, user_id_arg, permission_id],
    );

    let tx = ptb.finish();

    client.execute_transaction(tx, gas_budget).await?;

    let federation_operations = client.onchain(federation_id);

    if federation_operations
      .has_permission_to_attest(user_id)
      .await
      .context("failed to check if federation has property")?
    {
      anyhow::bail!("failed to revoke permission to accredit");
    }

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

    let tx_res = client.execute_transaction(tx, gas_budget).await?;

    if !tx_res.status_ok().ok_or_else(|| anyhow::anyhow!("missing status"))? {
      anyhow::bail!("failed to add root authority");
    }

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
    let want_property_constraints = ptb.pure(want_property_constraints)?;

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("issue_permission_to_accredit").into(),
      vec![],
      vec![cap, fed_ref, receiver_arg, want_property_constraints],
    );

    let tx = ptb.finish();

    let tx_res = client.execute_transaction(tx, gas_budget).await?;

    // check if the ID has AccreditCap
    if !tx_res.status_ok().ok_or_else(|| anyhow::anyhow!("missing status"))? {
      anyhow::bail!("failed to issue permission to accredit");
    }

    let Ok(_) = get_cap(client, "main", "AccreditCap", Some(receiver.into())).await else {
      anyhow::bail!("failed to get new accredit");
    };

    Ok(())
  }

  pub async fn validate_credential<S>(
    client: &HTFClient<S>,
    federation_id: ObjectID,
    credential_id: ObjectID,
    gas_budget: Option<u64>,
  ) -> anyhow::Result<()>
  where
    S: Signer<IotaKeySignature>,
  {
    let mut ptb = ProgrammableTransactionBuilder::new();

    let cred = client.get_object_ref_by_id(credential_id).await?;

    let cred = ptb.obj(ObjectArg::ImmOrOwnedObject(cred))?;
    let fed_ref = ptb.obj(ObjectArg::SharedObject {
      id: federation_id,
      initial_shared_version: client.initial_shared_version(&federation_id).await?,
      mutable: false,
    })?;

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("validate_credential").into(),
      vec![],
      vec![cred, fed_ref],
    );

    let tx = ptb.finish();

    if !client
      .execute_transaction(tx, gas_budget)
      .await?
      .status_ok()
      .ok_or_else(|| anyhow::anyhow!("Transaction failed"))?
    {
      return Err(anyhow::anyhow!("Transaction failed"));
    }

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
    let want_property_constraints = ptb.pure(want_property_constraints)?;

    ptb.programmable_move_call(
      client.htf_package_id(),
      ident_str!("main").into(),
      ident_str!("issue_permission_to_accredit").into(),
      vec![],
      vec![cap, fed_ref, receiver_arg, want_property_constraints],
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
      vec![cap, fed_ref, user_id_arg, permission_id],
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
    let cap_tag = StructTag::from_str(&format!("{}::{module}::{cap_type}", client.htf_package_id()))?;

    let filter = IotaObjectResponseQuery::new_with_filter(IotaObjectDataFilter::StructType(cap_tag));

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
