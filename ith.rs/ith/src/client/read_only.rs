use std::ops::Deref;

use anyhow::Context;
use iota_sdk::rpc_types::{IotaData, IotaObjectDataOptions};
use iota_sdk::types::base_types::{ObjectID, ObjectRef, SequenceNumber};
use iota_sdk::types::object::Owner;
use iota_sdk::IotaClient;

use crate::provider::{OffChainFederation, OnChainFederation};

/// A read-only client for the ITH.
///
/// This client is used for communicating with the ITH in a read-only manner.
///
/// This client supports both OffChain and OnChain operations on the ITH.
#[derive(Clone)]
pub struct ITHClientReadOnly {
  client: IotaClient,
  ith_package_id: ObjectID,
}

impl ITHClientReadOnly {
  /// Creates a new read-only client for the ITH.
  pub fn new(client: IotaClient, ith_package_id: ObjectID) -> Self {
    Self {
      client,
      ith_package_id,
    }
  }

  /// Returns the ITH package ID.
  pub fn ith_package_id(&self) -> ObjectID {
    self.ith_package_id
  }

  /// Returns the underlying Iota client.
  pub fn client(&self) -> &IotaClient {
    &self.client
  }

  /// Performs off-chain operations on the ITH.
  pub async fn offchain(&self, federation_id: ObjectID) -> anyhow::Result<OffChainFederation> {
    OffChainFederation::new(self, federation_id).await
  }

  pub fn onchain(&self, federation_id: ObjectID) -> OnChainFederation {
    OnChainFederation::new(self, federation_id)
  }

  /// Returns an object by its ID.
  pub async fn get_object_by_id<R>(&self, id: ObjectID) -> anyhow::Result<R>
  where
    R: serde::de::DeserializeOwned,
  {
    let res = self
      .client
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

  pub(crate) async fn initial_shared_version(
    &self,
    object_id: &ObjectID,
  ) -> anyhow::Result<SequenceNumber> {
    let owner = self
      .read_api()
      .get_object_with_options(*object_id, IotaObjectDataOptions::default().with_owner())
      .await?
      .owner()
      .context("missing owner information")?;

    match owner {
      Owner::Shared {
        initial_shared_version,
      } => Ok(initial_shared_version),
      _ => anyhow::bail!(format!("object {object_id} is not a shared object")),
    }
  }

  #[allow(dead_code)]
  pub(crate) async fn get_object_ref_by_id(&self, obj: ObjectID) -> anyhow::Result<ObjectRef> {
    let res = self
      .read_api()
      .get_object_with_options(obj, IotaObjectDataOptions::new().with_content())
      .await?;

    let Some(data) = res.data else {
      return Err(anyhow::anyhow!("no data found"));
    };

    Ok(data.object_ref())
  }
}

impl Deref for ITHClientReadOnly {
  type Target = IotaClient;

  fn deref(&self) -> &Self::Target {
    &self.client
  }
}
