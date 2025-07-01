//! Client module provides the client interface for the ITH service.
//! Clients can be used to interact with the ITH service, create new federations,
//! add trusted properties, create attestations, and accreditations.
//!
//! There are two types of clients:
//! - Client: A client that can perform both on-chain and off-chain operations. It requires a signer with a private key.
//!   The client is represented by the [`ITHClient`] struct.
//! - ReadOnlyClient: A client that can only perform off-chain operations. It doesn't require a signer with a private
//!   key. The client is represented by the [`ITHClientReadOnly`] struct.
mod full_client;
mod read_only;

pub use full_client::*;
use iota_interaction::IotaClientTrait;
use iota_interaction_rust::IotaClientAdapter;
use iota_sdk::rpc_types::{IotaData, IotaObjectDataOptions};
use iota_sdk::types::base_types::ObjectID;
use product_common::core_client::CoreClientReadOnly;
use product_common::network_name::NetworkName;
pub use read_only::*;
use serde::de::DeserializeOwned;

use crate::error::Error;

/// Returns the network-id also known as chain-identifier provided by the specified iota_client
async fn network_id(iota_client: &IotaClientAdapter) -> Result<NetworkName, Error> {
    let network_id = iota_client
        .read_api()
        .get_chain_identifier()
        .await
        .map_err(|e| Error::RpcError(e.to_string()))?;
    Ok(network_id.try_into().expect("chain ID is a valid network name"))
}

pub async fn get_object_ref_by_id_with_bcs<T: DeserializeOwned>(
    client: &impl CoreClientReadOnly,
    object_id: &ObjectID,
) -> Result<T, Error> {
    let notarization = client
        .client_adapter()
        .read_api()
        .get_object_with_options(*object_id, IotaObjectDataOptions::bcs_lossless())
        .await
        .map_err(|err| Error::ObjectLookup(err.to_string()))?
        .data
        .ok_or_else(|| Error::ObjectLookup("missing data in response".to_string()))?
        .bcs
        .ok_or_else(|| Error::ObjectLookup("missing object content in data".to_string()))?
        .try_into_move()
        .ok_or_else(|| Error::ObjectLookup("failed to convert data to move object".to_string()))?
        .deserialize()
        .map_err(|err| Error::ObjectLookup(err.to_string()))?;

    Ok(notarization)
}
