use anyhow::Context;
use iota_sdk::rpc_types::{IotaData, IotaObjectDataOptions};
use iota_sdk::types::base_types::{ObjectID, ObjectRef};
use iota_sdk::types::collection_types::{Table, VecMap, VecSet};
use iota_sdk::types::id::{ID, UID};
use serde::{Deserialize, Serialize};

use crate::client::HTFClient;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthorityCap {
    id: UID,
    federation_id: ID,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustService {
    id: UID,
    service_type: String,
    data: Vec<TrustServiceProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustServiceProperty {
    name: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthority {
    pub id: UID,
    pub account_id: ID,
    permissions: Permissions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Governance {
    id: UID,
    trusted_constraints: TrustedPropertyConstraints,
    accreditors: VecMap<ID, PermissionsToAccredit>,
    attesters: VecMap<ID, PermissionsToAttest>,
    credentials_state: VecMap<ID, CredentialState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionsToAccredit {
    permissions: Vec<PermissionToAccredit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionToAccredit {
    id: UID,
    federation_id: ID,
    created_by: String,
    constraints: VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionsToAttest {
    permissions: Vec<PermissionToAttest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// PermissionToAttest can be created only by the HTF module
pub struct PermissionToAttest {
    id: UID,
    federation_id: ID,
    created_by: String,
    constraints: VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialState {
    is_revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    //Table<ID, vector<Attestation>>
    attestations: VecMap<ID, Vec<PermissionToAttest>>,
    // Table<ID, vector<Accreditation>>
    permissions_to_accredit: VecMap<ID, Vec<PermissionToAccredit>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accreditation {
    id: UID,
    created_by: String,
    // Table<TrustedPropertyName, TrustedPropertyConstraint>
    trusted_properties: Table,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    id: UID,
    created_by: String,
    // Table<TrustedPropertyName, TrustedPropertyConstraint>
    trusted_properties: Table,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedPropertyName {
    // initially its a string, but it could be more complex structure that implements copy and drop
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedPropertyValue {
    // initially its a string, but it could be more complex structure that implements copy and drop
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    pub id: UID,
    pub issued_by: ID,
    pub issued_for: ID,
    pub valid_from: u64,
    pub valid_to: u64,
    pub trusted_properties: VecMap<TrustedPropertyName, TrustedPropertyValue>,
}

impl Credential {
    pub async fn get_by_id(id: ObjectID, client: &HTFClient) -> anyhow::Result<Self> {
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

    pub async fn get_object_reference(id: UID, client: &HTFClient) -> anyhow::Result<ObjectRef> {
        let res = client
            .read_api()
            .get_object_with_options(*id.object_id(), IotaObjectDataOptions::new().with_content())
            .await?;

        let Some(data) = res.data else {
            return Err(anyhow::anyhow!("no data"));
        };

        Ok(data.object_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub struct TrustedPropertyConstraints {
    data: VecMap<TrustedPropertyName, TrustedPropertyConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedPropertyConstraint {
    property_name: TrustedPropertyName,
    // allow only set of values
    allowed_values: VecSet<TrustedPropertyValue>,
    // allow_any - takes a precedence over the allowed_values
    allow_any: bool,
}
