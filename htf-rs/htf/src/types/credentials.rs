use std::collections::HashMap;

use iota_sdk::rpc_types::IotaObjectDataOptions;
use iota_sdk::types::base_types::ObjectRef;
use iota_sdk::types::id::{ID, UID};
use serde::{Deserialize, Serialize};

use super::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use crate::client::HTFClient;
use crate::de::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialState {
  is_revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
  pub id: UID,
  pub issued_by: ID,
  pub issued_for: ID,
  pub valid_from: u64,
  pub valid_to: u64,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub trusted_properties: HashMap<TrustedPropertyName, TrustedPropertyValue>,
}

impl Credential {
  pub async fn get_object_reference<S>(id: UID, client: &HTFClient<S>) -> anyhow::Result<ObjectRef> {
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
