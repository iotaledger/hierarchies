// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Shared utilities for Access Controller Bridge examples.

use anyhow::{Context, anyhow};
use hierarchies::core::types::property_name::PropertyName;
use iota_interaction::types::base_types::{ObjectID, ObjectRef};
use iota_interaction::types::object::Owner;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::{
    Argument, Command, ObjectArg, Transaction, TransactionData,
};
use iota_interaction::types::TypeTag;
use iota_interaction::types::{Identifier, IOTA_CLOCK_OBJECT_ID, IOTA_CLOCK_OBJECT_SHARED_VERSION};
use iota_interaction::{MoveType, ident_str};
use iota_sdk::rpc_types::{
    IotaObjectDataOptions, IotaTransactionBlockResponseOptions, ObjectChange,
};
use iota_sdk::{IOTA_LOCAL_NETWORK_URL, IotaClient, IotaClientBuilder};
use product_common::test_utils::{InMemSigner, request_funds};
use secret_storage::Signer;
use std::str::FromStr;

// ===== Move names: Hierarchies =====

mod hier {
    pub const MOD_MAIN: &str = "main";
    pub const MOD_PROPERTY: &str = "property";
    pub const MOD_PROPERTY_VALUE: &str = "property_value";
    pub const MOD_PROPERTY_SHAPE: &str = "property_shape";
    pub const MOD_UTILS: &str = "utils";

    pub const FN_NEW_FEDERATION: &str = "new_federation";
    pub const FN_ADD_PROPERTY: &str = "add_property";
    pub const FN_CREATE_ACCREDITATION_TO_ATTEST: &str = "create_accreditation_to_attest";
    pub const FN_NEW_PROPERTY: &str = "new_property";
    pub const FN_NEW_PROPERTY_VALUE_STRING: &str = "new_property_value_string";
    pub const FN_CREATE_VEC_SET: &str = "create_vec_set";
    pub const FN_VEC_MAP_FROM_KEYS_VALUES: &str = "vec_map_from_keys_values";

    pub const TYPE_PROPERTY_VALUE: &str = "PropertyValue";
    pub const TYPE_PROPERTY_SHAPE: &str = "PropertyShape";
    pub const TYPE_FEDERATION_PROPERTY: &str = "FederationProperty";
}

// ===== Move names: Audit Trail =====

mod trail {
    pub const MOD_MAIN: &str = "main";
    pub const MOD_RECORD: &str = "record";
    pub const MOD_LOCKING: &str = "locking";
    pub const MOD_PERMISSION: &str = "permission";
    pub const MOD_RECORD_TAGS: &str = "record_tags";

    pub const FN_CREATE: &str = "create";
    pub const FN_CREATE_ROLE: &str = "create_role";
    pub const FN_NEW_CAPABILITY: &str = "new_capability";
    pub const FN_ADD_RECORD: &str = "add_record";
    pub const FN_NEW_TRAIL_METADATA: &str = "new_trail_metadata";
    pub const FN_NEW_TEXT: &str = "new_text";
    pub const FN_WINDOW_NONE: &str = "window_none";
    pub const FN_NEW: &str = "new";
    pub const FN_FROM_VEC: &str = "from_vec";

    pub const TYPE_DATA: &str = "Data";
    pub const TYPE_INITIAL_RECORD: &str = "InitialRecord";
    pub const TYPE_IMMUTABLE_METADATA: &str = "ImmutableMetadata";
    pub const TYPE_PERMISSION: &str = "Permission";
    pub const TYPE_ROLE_TAGS: &str = "RoleTags";
}

// ===== Move names: ACB =====

mod acb_names {
    pub const MOD_BRIDGE: &str = "bridge";

    pub const FN_CREATE: &str = "create";
    pub const FN_DEPOSIT_CAPABILITY: &str = "deposit_capability";
    pub const FN_BORROW: &str = "borrow";
    pub const FN_RETURN_CAP: &str = "return_cap";
    pub const FN_NEW_ROLE_CONFIG: &str = "new_role_config";
    pub const FN_ROLE_NAME: &str = "role_name";

    pub const TYPE_ROLE_CONFIG: &str = "RoleConfig";
    pub const TYPE_PERMISSION_CONTEXT: &str = "PermissionContext";
    pub const TYPE_ACB: &str = "AccessControllerBridge";
}

// ===== Move names: TfComponents =====

mod tf {
    pub const MOD_TIMELOCK: &str = "timelock";
    pub const FN_NONE: &str = "none";
}

// ===== Move names: Framework =====

const MOVE_STDLIB: &str = "0x1";
const IOTA_FRAMEWORK: &str = "0x2";
const MOD_OPTION: &str = "option";
const MOD_TRANSFER: &str = "transfer";
const FN_OPTION_NONE: &str = "none";
const FN_OPTION_SOME: &str = "some";
const FN_PUBLIC_SHARE_OBJECT: &str = "public_share_object";
const TYPE_STRING: &str = "0x1::string::String";

// ===== Environment =====

pub fn env_pkg(name: &str) -> anyhow::Result<ObjectID> {
    std::env::var(name)
        .map_err(|e| anyhow!("env variable {name} must be set").context(e))
        .and_then(|s| s.parse().context("invalid package id"))
}

pub fn hierarchies_pkg() -> anyhow::Result<ObjectID> { env_pkg("IOTA_HIERARCHIES_PKG_ID") }
pub fn audit_trail_pkg() -> anyhow::Result<ObjectID> { env_pkg("IOTA_AUDIT_TRAIL_PKG_ID") }
pub fn acb_pkg() -> anyhow::Result<ObjectID> { env_pkg("IOTA_ACB_PKG_ID") }
pub fn tf_components_pkg() -> anyhow::Result<ObjectID> { env_pkg("IOTA_TF_COMPONENTS_PKG_ID") }

// ===== Client setup =====

async fn iota_client() -> anyhow::Result<IotaClient> {
    let url = std::env::var("API_ENDPOINT").unwrap_or_else(|_| IOTA_LOCAL_NETWORK_URL.to_string());
    IotaClientBuilder::default().build(&url).await.map_err(|e| anyhow!("connect failed: {e}"))
}

pub async fn get_client_and_signer() -> anyhow::Result<(IotaClient, InMemSigner)> {
    let signer = InMemSigner::new();
    let addr = signer.get_address().await?;
    request_funds(&addr).await?;
    Ok((iota_client().await?, signer))
}

// ===== Object refs =====

pub async fn shared_arg(client: &IotaClient, id: ObjectID, mutable: bool) -> anyhow::Result<ObjectArg> {
    let resp = client.read_api()
        .get_object_with_options(id, IotaObjectDataOptions::default().with_owner())
        .await?;
    let owner = resp.owner().ok_or_else(|| anyhow!("object {id} not found"))?;
    match owner {
        Owner::Shared { initial_shared_version } =>
            Ok(ObjectArg::SharedObject { id, initial_shared_version, mutable }),
        _ => Err(anyhow!("object {id} is not shared")),
    }
}

pub async fn owned_ref(client: &IotaClient, id: ObjectID) -> anyhow::Result<ObjectRef> {
    let resp = client.read_api()
        .get_object_with_options(id, IotaObjectDataOptions::new())
        .await?;
    Ok(resp.data.ok_or_else(|| anyhow!("object {id} not found"))?.object_ref())
}

pub fn clock_arg(ptb: &mut ProgrammableTransactionBuilder) -> Argument {
    ptb.obj(ObjectArg::SharedObject {
        id: IOTA_CLOCK_OBJECT_ID,
        initial_shared_version: IOTA_CLOCK_OBJECT_SHARED_VERSION,
        mutable: false,
    }).unwrap()
}

// ===== PTB helper =====

pub struct PtbHelper {
    pub hier_pkg: ObjectID,
    pub at_pkg: ObjectID,
    pub acb_pkg: ObjectID,
    pub tf_pkg: ObjectID,
}

impl PtbHelper {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            hier_pkg: hierarchies_pkg()?,
            at_pkg: audit_trail_pkg()?,
            acb_pkg: acb_pkg()?,
            tf_pkg: tf_components_pkg()?,
        })
    }

    // -- Type tags --

    fn tt(&self, pkg: ObjectID, module: &str, name: &str) -> TypeTag {
        TypeTag::from_str(&format!("{pkg}::{module}::{name}")).unwrap()
    }

    pub fn data_tag(&self) -> TypeTag {
        self.tt(self.at_pkg, trail::MOD_RECORD, trail::TYPE_DATA)
    }

    pub fn acb_marker(&self) -> TypeTag {
        TypeTag::from_str("bool").unwrap()
    }

    // -- Federation: new_federation --

    pub fn new_federation(&self, ptb: &mut ProgrammableTransactionBuilder) {
        ptb.programmable_move_call(
            self.hier_pkg,
            ident_str!(hier::MOD_MAIN).into(),
            ident_str!(hier::FN_NEW_FEDERATION).into(),
            vec![], vec![],
        );
    }

    // -- Property construction --

    pub fn prop_name(&self, ptb: &mut ProgrammableTransactionBuilder, name: &str) -> anyhow::Result<Argument> {
        PropertyName::from(name).to_ptb(ptb, self.hier_pkg)
    }

    pub fn prop_value_str(&self, ptb: &mut ProgrammableTransactionBuilder, val: &str) -> anyhow::Result<Argument> {
        let s = ptb.pure(val.to_string())?;
        Ok(ptb.programmable_move_call(
            self.hier_pkg,
            ident_str!(hier::MOD_PROPERTY_VALUE).into(),
            ident_str!(hier::FN_NEW_PROPERTY_VALUE_STRING).into(),
            vec![], vec![s],
        ))
    }

    /// Build a `FederationProperty` argument in the PTB.
    fn fed_property(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        name: &str,
        values: &[&str],
        allow_any: bool,
    ) -> anyhow::Result<Argument> {
        let pn = self.prop_name(ptb, name)?;
        let val_tag = self.tt(self.hier_pkg, hier::MOD_PROPERTY_VALUE, hier::TYPE_PROPERTY_VALUE);
        let val_args: Vec<_> = values.iter()
            .map(|v| self.prop_value_str(ptb, v))
            .collect::<Result<_, _>>()?;
        let vv = ptb.command(Command::MakeMoveVec(Some(val_tag.clone().into()), val_args));
        let aset = ptb.programmable_move_call(
            self.hier_pkg,
            ident_str!(hier::MOD_UTILS).into(),
            ident_str!(hier::FN_CREATE_VEC_SET).into(),
            vec![val_tag], vec![vv],
        );
        let aa = ptb.pure(allow_any)?;
        let shape_tag = self.tt(self.hier_pkg, hier::MOD_PROPERTY_SHAPE, hier::TYPE_PROPERTY_SHAPE);
        let sn = option_none(ptb, shape_tag)?;
        Ok(ptb.programmable_move_call(
            self.hier_pkg,
            ident_str!(hier::MOD_PROPERTY).into(),
            ident_str!(hier::FN_NEW_PROPERTY).into(),
            vec![], vec![pn, aset, aa, sn],
        ))
    }

    // -- VecMap<PropertyName, PropertyValue> --

    pub fn property_map(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        pairs: Vec<(&str, &str)>,
    ) -> anyhow::Result<Argument> {
        let mut keys = vec![];
        let mut vals = vec![];
        for (k, v) in pairs {
            keys.push(self.prop_name(ptb, k)?);
            vals.push(self.prop_value_str(ptb, v)?);
        }
        let kt = PropertyName::move_type(self.hier_pkg);
        let vt = self.tt(self.hier_pkg, hier::MOD_PROPERTY_VALUE, hier::TYPE_PROPERTY_VALUE);
        Ok(self.vec_map(ptb, kt, vt, keys, vals))
    }

    fn vec_map(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        kt: TypeTag,
        vt: TypeTag,
        keys: Vec<Argument>,
        vals: Vec<Argument>,
    ) -> Argument {
        let kv = ptb.command(Command::MakeMoveVec(Some(kt.clone().into()), keys));
        let vv = ptb.command(Command::MakeMoveVec(Some(vt.clone().into()), vals));
        ptb.programmable_move_call(
            self.hier_pkg,
            ident_str!(hier::MOD_UTILS).into(),
            ident_str!(hier::FN_VEC_MAP_FROM_KEYS_VALUES).into(),
            vec![kt, vt], vec![kv, vv],
        )
    }

    // -- Federation: add_property --

    pub fn fed_add_property(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        fed_arg: Argument,
        cap_arg: Argument,
        name: &str,
        values: Vec<&str>,
        allow_any: bool,
    ) -> anyhow::Result<()> {
        let prop = self.fed_property(ptb, name, &values, allow_any)?;
        ptb.programmable_move_call(
            self.hier_pkg,
            ident_str!(hier::MOD_MAIN).into(),
            ident_str!(hier::FN_ADD_PROPERTY).into(),
            vec![], vec![fed_arg, cap_arg, prop],
        );
        Ok(())
    }

    // -- Federation: accredit_to_attest --

    pub fn fed_accredit_to_attest(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        fed_arg: Argument,
        accredit_cap: Argument,
        receiver: iota_interaction::types::base_types::IotaAddress,
        props: Vec<(&str, Vec<&str>, bool)>,
    ) -> anyhow::Result<()> {
        let receiver_arg = ptb.pure(receiver)?;
        let prop_tag = self.tt(self.hier_pkg, hier::MOD_PROPERTY, hier::TYPE_FEDERATION_PROPERTY);
        let mut prop_args = vec![];
        for (name, values, allow_any) in &props {
            prop_args.push(self.fed_property(ptb, name, values, *allow_any)?);
        }
        let pv = ptb.command(Command::MakeMoveVec(Some(prop_tag.into()), prop_args));
        let clock = clock_arg(ptb);
        ptb.programmable_move_call(
            self.hier_pkg,
            ident_str!(hier::MOD_MAIN).into(),
            ident_str!(hier::FN_CREATE_ACCREDITATION_TO_ATTEST).into(),
            vec![], vec![fed_arg, accredit_cap, receiver_arg, pv, clock],
        );
        Ok(())
    }

    // -- ACB: create --

    /// Create an ACB with role configs.
    /// Each role is `(role_name, property_name_value_pairs)`.
    pub fn acb_create(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        federation_arg: Argument,
        target_id: ObjectID,
        roles: Vec<(&str, Vec<(&str, &str)>)>,
    ) -> anyhow::Result<Argument> {
        let target = ptb.pure(target_id)?;
        let mut keys = vec![];
        let mut vals = vec![];
        for (name, prop_pairs) in roles {
            keys.push(ptb.pure(name.to_string())?);
            let prop_map = self.property_map(ptb, prop_pairs)?;
            vals.push(ptb.programmable_move_call(
                self.acb_pkg,
                ident_str!(acb_names::MOD_BRIDGE).into(),
                ident_str!(acb_names::FN_NEW_ROLE_CONFIG).into(),
                vec![], vec![prop_map],
            ));
        }

        let str_tag = TypeTag::from_str(TYPE_STRING).unwrap();
        let cfg_tag = self.tt(self.acb_pkg, acb_names::MOD_BRIDGE, acb_names::TYPE_ROLE_CONFIG);
        let configs = self.vec_map(ptb, str_tag, cfg_tag, keys, vals);

        Ok(ptb.programmable_move_call(
            self.acb_pkg,
            ident_str!(acb_names::MOD_BRIDGE).into(),
            ident_str!(acb_names::FN_CREATE).into(),
            vec![self.acb_marker()], vec![federation_arg, target, configs],
        ))
    }

    pub fn share_acb(&self, ptb: &mut ProgrammableTransactionBuilder, acb_arg: Argument) {
        let iota_fw = ObjectID::from_hex_literal(IOTA_FRAMEWORK).unwrap();
        let acb_tt = TypeTag::from_str(
            &format!("{}::{}::{}<bool>", self.acb_pkg, acb_names::MOD_BRIDGE, acb_names::TYPE_ACB)
        ).unwrap();
        ptb.programmable_move_call(
            iota_fw,
            Identifier::new(MOD_TRANSFER).unwrap(),
            Identifier::new(FN_PUBLIC_SHARE_OBJECT).unwrap(),
            vec![acb_tt], vec![acb_arg],
        );
    }

    // -- ACB: deposit --

    pub fn acb_deposit(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        acb_arg: Argument,
        fed_arg: Argument,
        cap_type: &str,
        cap_arg: Argument,
    ) -> anyhow::Result<()> {
        let ct = ptb.pure(cap_type.to_string())?;
        ptb.programmable_move_call(
            self.acb_pkg,
            ident_str!(acb_names::MOD_BRIDGE).into(),
            ident_str!(acb_names::FN_DEPOSIT_CAPABILITY).into(),
            vec![self.acb_marker()], vec![acb_arg, fed_arg, ct, cap_arg],
        );
        Ok(())
    }

    // -- ACB: borrow --

    /// Borrow a capability by role name. Constructs PermissionContext::RoleName on-chain.
    pub fn acb_borrow(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        acb_arg: Argument,
        fed_arg: Argument,
        role: &str,
    ) -> anyhow::Result<(Argument, Argument)> {
        let name_arg = ptb.pure(role.to_string())?;
        let context = ptb.programmable_move_call(
            self.acb_pkg,
            ident_str!(acb_names::MOD_BRIDGE).into(),
            ident_str!(acb_names::FN_ROLE_NAME).into(),
            vec![], vec![name_arg],
        );
        let clock = clock_arg(ptb);
        let result = ptb.programmable_move_call(
            self.acb_pkg,
            ident_str!(acb_names::MOD_BRIDGE).into(),
            ident_str!(acb_names::FN_BORROW).into(),
            vec![self.acb_marker()], vec![acb_arg, fed_arg, context, clock],
        );
        let cap = match result { Argument::Result(i) => Argument::NestedResult(i, 0), _ => unreachable!() };
        let receipt = match result { Argument::Result(i) => Argument::NestedResult(i, 1), _ => unreachable!() };
        Ok((cap, receipt))
    }

    // -- ACB: return_cap --

    pub fn acb_return(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        acb_arg: Argument,
        cap: Argument,
        receipt: Argument,
    ) {
        let clock = clock_arg(ptb);
        ptb.programmable_move_call(
            self.acb_pkg,
            ident_str!(acb_names::MOD_BRIDGE).into(),
            ident_str!(acb_names::FN_RETURN_CAP).into(),
            vec![self.acb_marker()], vec![acb_arg, cap, receipt, clock],
        );
    }

    // -- Audit trail: create --

    pub fn trail_create(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        name: &str,
        sender: iota_interaction::types::base_types::IotaAddress,
    ) -> anyhow::Result<Argument> {
        let ir_tag = TypeTag::from_str(&format!(
            "{}::{}::{}<{}::{}::{}>",
            self.at_pkg, trail::MOD_RECORD, trail::TYPE_INITIAL_RECORD,
            self.at_pkg, trail::MOD_RECORD, trail::TYPE_DATA,
        ))?;
        let initial = option_none(ptb, ir_tag)?;

        let wn = ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_LOCKING).into(),
            ident_str!(trail::FN_WINDOW_NONE).into(),
            vec![], vec![],
        );
        let tl1 = ptb.programmable_move_call(
            self.tf_pkg,
            ident_str!(tf::MOD_TIMELOCK).into(),
            ident_str!(tf::FN_NONE).into(),
            vec![], vec![],
        );
        let tl2 = ptb.programmable_move_call(
            self.tf_pkg,
            ident_str!(tf::MOD_TIMELOCK).into(),
            ident_str!(tf::FN_NONE).into(),
            vec![], vec![],
        );
        let locking = ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_LOCKING).into(),
            ident_str!(trail::FN_NEW).into(),
            vec![], vec![wn, tl1, tl2],
        );

        let name_a = ptb.pure(name.to_string())?;
        let desc = ptb.pure(Option::<String>::None)?;
        let md = ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_MAIN).into(),
            ident_str!(trail::FN_NEW_TRAIL_METADATA).into(),
            vec![], vec![name_a, desc],
        );
        let md_tag = self.tt(self.at_pkg, trail::MOD_MAIN, trail::TYPE_IMMUTABLE_METADATA);
        let trail_md = option_some(ptb, md_tag, md)?;
        let upd_none = ptb.pure(Option::<String>::None)?;
        let tags = ptb.pure(Vec::<String>::new())?;
        let clock = clock_arg(ptb);

        let result = ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_MAIN).into(),
            ident_str!(trail::FN_CREATE).into(),
            vec![self.data_tag()], vec![initial, locking, trail_md, upd_none, tags, clock],
        );

        let admin_cap = match result { Argument::Result(i) => Argument::NestedResult(i, 0), _ => unreachable!() };
        ptb.transfer_arg(sender, admin_cap);
        Ok(result)
    }

    // -- Audit trail: create_role --

    pub fn trail_create_role(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        trail_arg: Argument,
        admin_cap: Argument,
        role_name: &str,
        permission_fns: Vec<&str>,
    ) -> anyhow::Result<()> {
        let role = ptb.pure(role_name.to_string())?;
        let perm_tag = self.tt(self.at_pkg, trail::MOD_PERMISSION, trail::TYPE_PERMISSION);
        let perms: Vec<_> = permission_fns.iter().map(|f| {
            ptb.programmable_move_call(
                self.at_pkg,
                ident_str!(trail::MOD_PERMISSION).into(),
                Identifier::new(*f).unwrap(),
                vec![], vec![],
            )
        }).collect();
        let pv = ptb.command(Command::MakeMoveVec(Some(perm_tag.into()), perms));
        let ps = ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_PERMISSION).into(),
            ident_str!(trail::FN_FROM_VEC).into(),
            vec![], vec![pv],
        );
        let rt_tag = self.tt(self.at_pkg, trail::MOD_RECORD_TAGS, trail::TYPE_ROLE_TAGS);
        let rt_none = option_none(ptb, rt_tag)?;
        let clock = clock_arg(ptb);
        ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_MAIN).into(),
            ident_str!(trail::FN_CREATE_ROLE).into(),
            vec![self.data_tag()], vec![trail_arg, admin_cap, role, ps, rt_none, clock],
        );
        Ok(())
    }

    // -- Audit trail: new_capability --

    pub fn trail_mint_capability(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        trail_arg: Argument,
        admin_cap: Argument,
        role_name: &str,
    ) -> anyhow::Result<()> {
        let role = ptb.pure(role_name.to_string())?;
        let n1 = ptb.pure(Option::<Vec<u8>>::None)?;
        let n2 = ptb.pure(Option::<u64>::None)?;
        let n3 = ptb.pure(Option::<u64>::None)?;
        let clock = clock_arg(ptb);
        ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_MAIN).into(),
            ident_str!(trail::FN_NEW_CAPABILITY).into(),
            vec![self.data_tag()], vec![trail_arg, admin_cap, role, n1, n2, n3, clock],
        );
        Ok(())
    }

    // -- Audit trail: add_record --

    pub fn trail_add_record(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        trail_arg: Argument,
        cap: Argument,
        text: &str,
    ) -> anyhow::Result<()> {
        let t = ptb.pure(text.to_string())?;
        let data = ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_RECORD).into(),
            ident_str!(trail::FN_NEW_TEXT).into(),
            vec![], vec![t],
        );
        let md = ptb.pure(Option::<String>::None)?;
        let tag = ptb.pure(Option::<String>::None)?;
        let clock = clock_arg(ptb);
        ptb.programmable_move_call(
            self.at_pkg,
            ident_str!(trail::MOD_MAIN).into(),
            ident_str!(trail::FN_ADD_RECORD).into(),
            vec![self.data_tag()], vec![trail_arg, cap, data, md, tag, clock],
        );
        Ok(())
    }
}

// ===== Transaction execution =====

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
    let tx_data = TransactionData::new_programmable(sender, vec![gas_coin], tx, 100_000_000, gas_price);
    let sig = signer.sign(&tx_data).await?;

    let resp = client.quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(tx_data, vec![sig]),
            IotaTransactionBlockResponseOptions::new().with_effects().with_object_changes(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        ).await?;

    let effects = resp.effects.context("no effects")?;
    if effects.status().is_err() {
        return Err(anyhow!("transaction failed: {:?}", effects.status()));
    }
    Ok(resp.object_changes.unwrap_or_default())
}

pub fn find_created(changes: &[ObjectChange], type_contains: &str) -> Vec<ObjectID> {
    changes.iter().filter_map(|c| match c {
        ObjectChange::Created { object_id, object_type, .. }
            if object_type.to_string().contains(type_contains) => Some(*object_id),
        _ => None,
    }).collect()
}

// ===== Private helpers =====

fn option_none(ptb: &mut ProgrammableTransactionBuilder, tag: TypeTag) -> anyhow::Result<Argument> {
    Ok(ptb.programmable_move_call(
        ObjectID::from_hex_literal(MOVE_STDLIB).unwrap(),
        ident_str!(MOD_OPTION).into(),
        ident_str!(FN_OPTION_NONE).into(),
        vec![tag], vec![],
    ))
}

fn option_some(ptb: &mut ProgrammableTransactionBuilder, tag: TypeTag, val: Argument) -> anyhow::Result<Argument> {
    Ok(ptb.programmable_move_call(
        ObjectID::from_hex_literal(MOVE_STDLIB).unwrap(),
        ident_str!(MOD_OPTION).into(),
        ident_str!(FN_OPTION_SOME).into(),
        vec![tag], vec![val],
    ))
}
