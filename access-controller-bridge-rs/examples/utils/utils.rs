// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Shared utilities for Access Controller Bridge examples.

use anyhow::{Context, anyhow};
use iota_interaction::types::base_types::{ObjectID, ObjectRef};
use iota_interaction::types::object::Owner;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::{Argument, Command, ObjectArg, Transaction, TransactionData};
use iota_interaction::types::TypeTag;
use iota_interaction::types::{IOTA_CLOCK_OBJECT_ID, IOTA_CLOCK_OBJECT_SHARED_VERSION};
use iota_interaction::ident_str;
use iota_sdk::rpc_types::{
    IotaObjectDataOptions, IotaTransactionBlockResponseOptions, ObjectChange,
};
use iota_sdk::{IOTA_LOCAL_NETWORK_URL, IotaClient, IotaClientBuilder};
use product_common::test_utils::{InMemSigner, request_funds};
use secret_storage::Signer;
use std::str::FromStr;

// ===== Environment Variables =====

pub fn env_pkg(name: &str) -> anyhow::Result<ObjectID> {
    std::env::var(name)
        .map_err(|e| anyhow!("env variable {name} must be set").context(e))
        .and_then(|s| s.parse().context("invalid package id"))
}

pub fn hierarchies_pkg() -> anyhow::Result<ObjectID> { env_pkg("IOTA_HIERARCHIES_PKG_ID") }
pub fn audit_trail_pkg() -> anyhow::Result<ObjectID> { env_pkg("IOTA_AUDIT_TRAIL_PKG_ID") }
pub fn acb_pkg() -> anyhow::Result<ObjectID> { env_pkg("IOTA_ACB_PKG_ID") }
pub fn tf_components_pkg() -> anyhow::Result<ObjectID> { env_pkg("IOTA_TF_COMPONENTS_PKG_ID") }

// ===== Client Setup =====

pub async fn get_iota_client() -> anyhow::Result<IotaClient> {
    let url = std::env::var("API_ENDPOINT").unwrap_or_else(|_| IOTA_LOCAL_NETWORK_URL.to_string());
    IotaClientBuilder::default()
        .build(&url)
        .await
        .map_err(|e| anyhow!("failed to connect: {e}"))
}

pub async fn get_funded_signer() -> anyhow::Result<InMemSigner> {
    let signer = InMemSigner::new();
    let addr = signer.get_address().await?;
    request_funds(&addr).await?;
    Ok(signer)
}

// ===== Object References =====

pub async fn shared_obj_arg(client: &IotaClient, id: ObjectID, mutable: bool) -> anyhow::Result<ObjectArg> {
    let resp = client
        .read_api()
        .get_object_with_options(id, IotaObjectDataOptions::default().with_owner())
        .await?;
    let owner = resp.owner().ok_or_else(|| anyhow!("object {id} not found"))?;
    let isv = match owner {
        Owner::Shared { initial_shared_version } => initial_shared_version,
        _ => return Err(anyhow!("object {id} is not shared")),
    };
    Ok(ObjectArg::SharedObject { id, initial_shared_version: isv, mutable })
}

pub async fn owned_obj_ref(client: &IotaClient, id: ObjectID) -> anyhow::Result<ObjectRef> {
    let resp = client
        .read_api()
        .get_object_with_options(id, IotaObjectDataOptions::new())
        .await?;
    let data = resp.data.ok_or_else(|| anyhow!("object {id} not found"))?;
    Ok(data.object_ref())
}

pub fn clock_arg(ptb: &mut ProgrammableTransactionBuilder) -> Argument {
    ptb.obj(ObjectArg::SharedObject {
        id: IOTA_CLOCK_OBJECT_ID,
        initial_shared_version: IOTA_CLOCK_OBJECT_SHARED_VERSION,
        mutable: false,
    })
    .unwrap()
}

// ===== PTB Helpers: Property Types =====

pub fn ptb_property_name(
    ptb: &mut ProgrammableTransactionBuilder,
    hier_pkg: ObjectID,
    name: &str,
) -> anyhow::Result<Argument> {
    let s = ptb.pure(name.to_string())?;
    Ok(ptb.programmable_move_call(
        hier_pkg,
        ident_str!("property_name").into(),
        ident_str!("new_property_name").into(),
        vec![],
        vec![s],
    ))
}

pub fn ptb_property_value_string(
    ptb: &mut ProgrammableTransactionBuilder,
    hier_pkg: ObjectID,
    value: &str,
) -> anyhow::Result<Argument> {
    let s = ptb.pure(value.to_string())?;
    Ok(ptb.programmable_move_call(
        hier_pkg,
        ident_str!("property_value").into(),
        ident_str!("new_property_value_string").into(),
        vec![],
        vec![s],
    ))
}

// ===== PTB Helpers: VecMap via hierarchies::utils =====

pub fn ptb_vec_map_from_keys_values(
    ptb: &mut ProgrammableTransactionBuilder,
    hier_pkg: ObjectID,
    key_tag: TypeTag,
    val_tag: TypeTag,
    keys: Vec<Argument>,
    values: Vec<Argument>,
) -> Argument {
    let keys_vec = ptb.command(Command::MakeMoveVec(Some(key_tag.clone().into()), keys));
    let vals_vec = ptb.command(Command::MakeMoveVec(Some(val_tag.clone().into()), values));
    ptb.programmable_move_call(
        hier_pkg,
        ident_str!("utils").into(),
        ident_str!("vec_map_from_keys_values").into(),
        vec![key_tag, val_tag],
        vec![keys_vec, vals_vec],
    )
}

// ===== PTB Helpers: ACB =====

pub fn ptb_new_cap_type_config(
    ptb: &mut ProgrammableTransactionBuilder,
    hier_pkg: ObjectID,
    acb_pkg: ObjectID,
    property_name_args: Vec<Argument>,
) -> Argument {
    let name_tag = type_tag(hier_pkg, "property_name", "PropertyName");
    let props_vec = ptb.command(Command::MakeMoveVec(Some(name_tag.into()), property_name_args));
    ptb.programmable_move_call(
        acb_pkg,
        ident_str!("bridge").into(),
        ident_str!("new_capability_type_config").into(),
        vec![],
        vec![props_vec],
    )
}

// ===== Type Tags =====

pub fn type_tag(pkg: ObjectID, module: &str, name: &str) -> TypeTag {
    TypeTag::from_str(&format!("{pkg}::{module}::{name}")).unwrap()
}

pub fn string_type_tag() -> TypeTag {
    TypeTag::from_str("0x1::string::String").unwrap()
}

// ===== Transaction Execution =====

pub async fn execute_ptb(
    client: &IotaClient,
    signer: &InMemSigner,
    ptb: ProgrammableTransactionBuilder,
) -> anyhow::Result<Vec<ObjectChange>> {
    use iota_interaction::types::quorum_driver_types::ExecuteTransactionRequestType;
    use iota_sdk::rpc_types::IotaTransactionBlockEffectsAPI;

    let sender = signer.get_address().await?;
    let tx = ptb.finish();

    let coins = client.coin_read_api().get_coins(sender, None, None, None).await?;
    let gas_coin = coins.data.first().context("no gas coins")?.object_ref();
    let gas_price = client.read_api().get_reference_gas_price().await?;

    let tx_data = TransactionData::new_programmable(
        sender,
        vec![gas_coin],
        tx,
        100_000_000,
        gas_price,
    );

    let sig = signer.sign(&tx_data).await?;
    let resp = client
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(tx_data, vec![sig]),
            IotaTransactionBlockResponseOptions::new()
                .with_effects()
                .with_object_changes(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    let effects = resp.effects.context("no effects")?;
    if effects.status().is_err() {
        return Err(anyhow!("transaction failed: {:?}", effects.status()));
    }

    Ok(resp.object_changes.unwrap_or_default())
}

pub fn find_created(changes: &[ObjectChange], type_contains: &str) -> Vec<ObjectID> {
    changes
        .iter()
        .filter_map(|c| match c {
            ObjectChange::Created { object_id, object_type, .. }
                if object_type.to_string().contains(type_contains) => Some(*object_id),
            _ => None,
        })
        .collect()
}
