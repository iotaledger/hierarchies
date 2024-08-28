use std::collections::HashMap;

use iota_sdk::rpc_types::IotaObjectDataOptions;
use iota_sdk::types::base_types::{ObjectID, ObjectRef};
use iota_sdk::types::id::UID;
use serde::{Deserialize, Serialize};

use super::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use crate::client::HTFClient;
use crate::utils::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialState {
  is_revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
  pub id: UID,
  pub issued_by: ObjectID,
  pub issued_for: ObjectID,
  pub valid_from: u64,
  pub valid_to: u64,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub trusted_properties: HashMap<TrustedPropertyName, TrustedPropertyValue>,
}

impl Credential {
  pub async fn get_object_reference<S>(id: ObjectID, client: &HTFClient<S>) -> anyhow::Result<ObjectRef> {
    let res = client
      .read_api()
      .get_object_with_options(id, IotaObjectDataOptions::new().with_content())
      .await?;

    let Some(data) = res.data else {
      anyhow::bail!("no data");
    };

    Ok(data.object_ref())
  }
}
