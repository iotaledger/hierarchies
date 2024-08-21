use std::str::FromStr;

use anyhow::Context;
use iota_sdk::rpc_types::{
    IotaData, IotaObjectDataFilter, IotaObjectDataOptions, IotaObjectResponseQuery,
    IotaTransactionBlockEffectsAPI,
};
use iota_sdk::types::base_types::{IotaAddress, ObjectID, ObjectRef, SequenceNumber};
use iota_sdk::types::collection_types::{VecMap, VecSet};
use iota_sdk::types::id::{ID, UID};
use iota_sdk::types::object::Owner;
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::{
    CallArg,  ObjectArg,  TransactionKind,
};
use iota_sdk::types::Identifier;
use move_core_types::language_storage::StructTag;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::client::HTFClient;

use crate::event::{Event, FederationCreatedEvent};

use crate::types::{
    Credential, Governance, RootAuthority, TrustedPropertyConstraints, TrustedPropertyName,
    TrustedPropertyValue,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Federation {
    id: UID,
    governance: Governance,
    pub root_authorities: Vec<RootAuthority>,
}

impl Federation {
    pub fn id(&self) -> ObjectID {
        *self.id.object_id()
    }

    async fn get_by_id(id: ObjectID, client: &HTFClient) -> anyhow::Result<Self> {
        let res = client
            .read_api()
            .get_object_with_options(id, IotaObjectDataOptions::new().with_content())
            .await?;

        let Some(data) = res.data else {
            return Err(anyhow::anyhow!("no data"));
        };

        let data = data
            .content
            .ok_or_else(|| anyhow::anyhow!("missing content"))
            .and_then(|content| content.try_into_move().context("invalid content"))
            .and_then(|data| {
                serde_json::from_value(data.fields.to_json_value()).context("invalid data")
            })?;

        Ok(data)
    }

    pub async fn get_federation_by_id(id: ObjectID, client: &HTFClient) -> anyhow::Result<Self> {
        Self::get_by_id(id, client).await
    }

    pub async fn create_new_federation(client: &HTFClient) -> anyhow::Result<Self> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        ptb.move_call(
            client.htf_package_id(),
            Identifier::from_str("main")?,
            Identifier::from_str("new_federation")?,
            vec![],
            vec![],
        )?;

        let tx = ptb.finish();

        let iota_tx = client.execute_transaction(tx).await?;

        println!("TX {:#?}", iota_tx);

        if !iota_tx
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            anyhow::bail!("failed to create federation, errors : {:?}", iota_tx.errors);
        }

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

        Self::get_by_id(fed_address.into(), client).await
    }

    async fn initial_shared_version(&self, client: &HTFClient) -> anyhow::Result<SequenceNumber> {
        let owner = client
            .read_api()
            .get_object_with_options(
                *self.id.object_id(),
                IotaObjectDataOptions::default().with_owner(),
            )
            .await?
            .owner()
            .context("missing owner information")?;

        match owner {
            Owner::Shared {
                initial_shared_version,
            } => Ok(initial_shared_version),
            _ => anyhow::bail!("`TransferProposal` is not a shared object"),
        }
    }

    pub async fn add_trusted_property(
        &self,
        client: &HTFClient,
        property_name: TrustedPropertyName,
        allowed_values: VecSet<TrustedPropertyValue>,
        allow_any: bool,
    ) -> anyhow::Result<()> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = get_cap("main", "RootAuthorityCap", None, client).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: true,
        })?;

        let property_name_arg = ptb.pure(&property_name)?;
        let allow_any = ptb.pure(allow_any)?;
        let allowed_values = ptb.pure(allowed_values)?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str("add_trusted_property").unwrap(),
            vec![],
            vec![cap, fed_ref, property_name_arg, allowed_values, allow_any],
        );

        let tx = ptb.finish();

        let res = client.execute_transaction(tx).await?;

        if !res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            let err = res.errors;
            anyhow::bail!("failed to add trusted property {:?}", err);
        }

        if !self
            .has_federation_property(client, &property_name)
            .await
            .context("failed to check if federation has property")?
        {
            anyhow::bail!("failed to add trusted property");
        }

        Ok(())
    }

    pub async fn add_root_authority(
        &self,
        client: &HTFClient,
        account_id: ID,
    ) -> anyhow::Result<()> {
        let cap = get_cap("main", "RootAuthorityCap", None, client).await?;

        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: true,
        })?;

        let account_id_arg = ptb.pure(&account_id)?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str("add_root_authority").unwrap(),
            vec![],
            vec![cap, fed_ref, account_id_arg],
        );

        let tx = ptb.finish();

        let tx_res = client.execute_transaction(tx).await?;

        if !tx_res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            anyhow::bail!("failed to add root authority");
        }

        let address: IotaAddress = account_id.bytes.into();

        let Ok(_) = get_cap("main", "RootAuthorityCap", Some(address), client).await else {
            anyhow::bail!("failed to get new authority");
        };

        Ok(())
    }

    pub async fn issue_permission_to_accredit(
        &self,
        client: &HTFClient,
        receiver: ID,
        want_property_constraints: Vec<TrustedPropertyConstraints>,
    ) -> anyhow::Result<()> {
        let cap = get_cap("main", "AccreditCap", None, client).await?;

        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: true,
        })?;

        let receiver_arg = ptb.pure(&receiver)?;
        let want_property_constraints = ptb.pure(want_property_constraints)?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str("issue_permission_to_accredit").unwrap(),
            vec![],
            vec![cap, fed_ref, receiver_arg, want_property_constraints],
        );

        let tx = ptb.finish();

        let tx_res = client.execute_transaction(tx).await?;

        // check if the ID has AccreditCap
        if !tx_res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            anyhow::bail!("failed to issue permission to accredit");
        }

        let Ok(_) = get_cap("main", "AccreditCap", Some(receiver.bytes.into()), client).await
        else {
            anyhow::bail!("failed to get new accredit");
        };

        if !self.has_permissions_to_accredit(client, receiver).await? {
            anyhow::bail!("failed to issue permission to accredit");
        }
        Ok(())
    }

    pub async fn issue_permission_to_attest(
        &self,
        client: &HTFClient,
        receiver: ID,
        want_property_constraints: Vec<TrustedPropertyConstraints>,
    ) -> anyhow::Result<()> {
        let cap = get_cap("main", "AttestCap", None, client).await?;

        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: true,
        })?;

        let receiver_arg = ptb.pure(&receiver)?;
        let want_property_constraints = ptb.pure(want_property_constraints)?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str("issue_permission_to_accredit").unwrap(),
            vec![],
            vec![cap, fed_ref, receiver_arg, want_property_constraints],
        );

        let tx = ptb.finish();

        let tx_res = client.execute_transaction(tx).await?;

        // check if the ID has AccreditCap
        if !tx_res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            anyhow::bail!("failed to issue permission to accredit");
        }

        let Ok(_) = get_cap("main", "AttestCap", Some(receiver.bytes.into()), client).await else {
            anyhow::bail!("failed to get new accredit");
        };

        if !self.has_permission_to_attest(client, receiver).await? {
            anyhow::bail!("failed to issue permission to accredit");
        }

        Ok(())
    }

    pub async fn revoke_permission_to_accredit(
        &self,
        client: &HTFClient,
        user_id: ID,
        permission_id: ID,
    ) -> anyhow::Result<()> {
        let cap = get_cap("main", "AccreditCap", None, client).await?;

        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: true,
        })?;

        let user_id_arg = ptb.pure(&user_id)?;
        let permission_id = ptb.pure(permission_id)?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str("revoke_permission_to_accredit").unwrap(),
            vec![],
            vec![cap, fed_ref, user_id_arg, permission_id],
        );

        let tx = ptb.finish();

        let iota_tx = client.execute_transaction(tx).await?;

        // check if the ID has AccreditCap
        if !iota_tx
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            anyhow::bail!("failed to revoke permission to accredit");
        }

        if self.has_permissions_to_accredit(client, user_id).await? {
            anyhow::bail!("failed to revoke permission to accredit");
        }

        Ok(())
    }

    pub async fn revoke_permission_to_attest(
        &self,
        client: &HTFClient,
        user_id: ID,
        permission_id: ID,
    ) -> anyhow::Result<()> {
        let cap = get_cap("main", "AttestCap", None, client).await?;

        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: true,
        })?;

        let user_id_arg = ptb.pure(&user_id)?;
        let permission_id = ptb.pure(permission_id)?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str("revoke_permission_to_accredit").unwrap(),
            vec![],
            vec![cap, fed_ref, user_id_arg, permission_id],
        );

        let tx = ptb.finish();

        let iota_res = client.execute_transaction(tx).await?;

        // check if the ID has AccreditCap
        if !iota_res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            anyhow::bail!("failed to revoke permission to accredit");
        }

        if self.has_permission_to_attest(client, user_id).await? {
            anyhow::bail!("failed to revoke permission to accredit");
        }

        Ok(())
    }

    pub async fn issue_credential(
        &self,
        client: &HTFClient,
        receiver: ID,
        trusted_properties: VecMap<TrustedPropertyName, TrustedPropertyValue>,
        valid_from_ts: u64,
        valid_until_ts: u64,
    ) -> anyhow::Result<()> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cap = get_cap("main", "AttestCap", None, client).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: true,
        })?;

        let receiver_arg = ptb.pure(&receiver)?;
        let trusted_properties_arg = ptb.pure(&trusted_properties)?;
        let valid_from_ts_arg = ptb.pure(valid_from_ts)?;
        let valid_until_ts_arg = ptb.pure(valid_until_ts)?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str("issue_credential").unwrap(),
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

        let iota_res = client.execute_transaction(tx).await?;

        // check if the ID has AccreditCap
        if !iota_res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            anyhow::bail!("failed to issue credential");
        }

        let created_object = iota_res
            .effects
            .ok_or_else(|| anyhow::anyhow!("missing effects"))?
            .created()
            .first()
            .ok_or_else(|| anyhow::anyhow!("missing created object"))?
            .object_id();

        let cred: Credential = get_object_by_id(created_object, client).await?;

        assert_eq!(cred.issued_for, receiver, "invalid issued_for");

        assert_eq!(cred.valid_from, valid_from_ts, "invalid valid_from");
        assert_eq!(cred.valid_to, valid_until_ts, "invalid valid_until");

        assert_eq!(
            cred.trusted_properties, trusted_properties,
            "invalid trusted_properties"
        );

        Ok(())
    }

    pub async fn validate_credential(
        &self,
        client: &HTFClient,
        credential_id: ID,
    ) -> anyhow::Result<()> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let cred = Credential::get_object_reference(UID { id: credential_id }, client)
            .await
            .unwrap();

        let cred = ptb.obj(ObjectArg::ImmOrOwnedObject(cred))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: false,
        })?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str("validate_credential").unwrap(),
            vec![],
            vec![cred, fed_ref],
        );

        let tx = ptb.finish();

        if !client
            .execute_transaction(tx)
            .await?
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("Transaction failed"))?
        {
            return Err(anyhow::anyhow!("Transaction failed"));
        }

        Ok(())
    }

    pub async fn has_permission_to_attest(
        &self,
        client: &HTFClient,
        user_id: ID,
    ) -> anyhow::Result<bool> {
        self.execute_query(client, "has_permission_to_attest", user_id)
            .await
    }

    pub async fn has_permissions_to_accredit(
        &self,
        client: &HTFClient,
        user_id: ID,
    ) -> anyhow::Result<bool> {
        self.execute_query(client, "has_permissions_to_accredit", user_id)
            .await
    }

    pub async fn has_federation_property(
        &self,
        client: &HTFClient,
        property_name: &TrustedPropertyName,
    ) -> anyhow::Result<bool> {
        self.execute_query(client, "has_federation_property", property_name)
            .await
    }

    async fn execute_query<T: Serialize, R: DeserializeOwned>(
        &self,
        client: &HTFClient,
        function_name: &str,
        arg: T,
    ) -> anyhow::Result<R> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        let arg = ptb.pure(arg)?;

        let fed_ref = ObjectArg::SharedObject {
            id: *self.id.object_id(),
            initial_shared_version: self.initial_shared_version(client).await?,
            mutable: false,
        };

        let arg = CallArg::Pure(bcs::to_bytes(&arg)?);
        let fed_ref = CallArg::Object(fed_ref);

        ptb.move_call(
            client.htf_package_id(),
            Identifier::from_str("main").unwrap(),
            Identifier::from_str(function_name).unwrap(),
            vec![],
            vec![fed_ref, arg],
        )?;

        let tx = TransactionKind::programmable(ptb.finish());

        let sender = client.sender_address();
        let return_values = client
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

async fn get_cap(
    module: &str,
    cap_type: &str,
    address: Option<IotaAddress>,
    client: &HTFClient,
) -> anyhow::Result<ObjectRef> {
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

pub async fn get_object_by_id<R>(id: ObjectID, client: &HTFClient) -> anyhow::Result<R>
where
    R: serde::de::DeserializeOwned,
{
    let res = client
        .read_api()
        .get_object_with_options(id, IotaObjectDataOptions::new().with_content())
        .await?;

    let Some(data) = res.data else {
        return Err(anyhow::anyhow!("no data"));
    };

    let data = data
        .content
        .ok_or_else(|| anyhow::anyhow!("missing content"))
        .and_then(|content| content.try_into_move().context("invalid content"))
        .and_then(|data| {
            serde_json::from_value(data.fields.to_json_value()).context("invalid data")
        })?;

    Ok(data)
}
