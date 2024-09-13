use std::collections::HashSet;
use std::str::FromStr;

use anyhow::Context;
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
use crate::types::trusted_constraints::TrustedPropertyConstraints;
use crate::types::trusted_property::{
    TrustedPropertyName, TrustedPropertyValue, TrustedPropertyValueMove,
};

pub(crate) mod ops {
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

        let cap = get_cap(client, "main", "RootAuthorityCap", None).await?;

        let cap = ptb.obj(ObjectArg::ImmOrOwnedObject(cap))?;
        let fed_ref = ptb.obj(ObjectArg::SharedObject {
            id: federation_id,
            initial_shared_version: client.initial_shared_version(&federation_id).await?,
            mutable: true,
        })?;

        let property_name_arg = ptb.pure(&property_name)?;
        let allow_any = ptb.pure(allow_any)?;
        let allowed_values = allowed_values
            .into_iter()
            .map(TrustedPropertyValueMove::from)
            .collect::<HashSet<_>>();
        let allowed_values = ptb.pure(allowed_values)?;

        ptb.programmable_move_call(
            client.htf_package_id(),
            ident_str!("main").into(),
            ident_str!("add_trusted_property").into(),
            vec![],
            vec![cap, fed_ref, property_name_arg, allowed_values, allow_any],
        );

        let tx = ptb.finish();

        let res = client.execute_transaction(tx, gas_budget).await?;

        if !res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            let err = res.errors;
            anyhow::bail!("failed to add trusted property {:?}", err);
        }

        let federation_operations = client.onchain(federation_id);

        if !federation_operations
            .has_federation_property(&property_name)
            .await
            .context("failed to check if federation has property")?
        {
            anyhow::bail!("failed to add trusted property");
        }

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
        let cap = get_cap(client, "main", "RootAuthorityCap", None).await?;

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
            vec![cap, fed_ref, account_id_arg],
        );

        let tx = ptb.finish();

        let tx_res = client.execute_transaction(tx, gas_budget).await?;

        if !tx_res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
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
        want_property_constraints: Vec<TrustedPropertyConstraints>,
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
        if !tx_res
            .status_ok()
            .ok_or_else(|| anyhow::anyhow!("missing status"))?
        {
            anyhow::bail!("failed to issue permission to accredit");
        }

        let Ok(_) = get_cap(client, "main", "AccreditCap", Some(receiver.into())).await else {
            anyhow::bail!("failed to get new accredit");
        };

        Ok(())
    }

    pub async fn issue_permission_to_attest<S>(
        client: &HTFClient<S>,
        federation_id: ObjectID,
        receiver: ObjectID,
        want_property_constraints: Vec<TrustedPropertyConstraints>,
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
