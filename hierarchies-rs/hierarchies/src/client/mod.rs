// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Client module provides the client interface for the Hierarchies service.
//! Clients can be used to interact with the Hierarchies service, create new federations,
//! add statements, create attestations, and accreditations.
//!
//! There are two types of clients:
//! - Client: A client that can perform both on-chain and off-chain operations. It requires a signer with a private key.
//!   The client is represented by the [`HierarchiesClient`] struct.
//! - ReadOnlyClient: A client that can only perform off-chain operations. It doesn't require a signer with a private
//!   key. The client is represented by the [`HierarchiesClientReadOnly`] struct.
pub mod error;
mod full_client;
mod read_only;

pub use error::ClientError;
pub use full_client::*;
use iota_interaction::rpc_types::{IotaData, IotaObjectDataOptions};
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::IotaClientTrait;
use product_common::core_client::CoreClientReadOnly;
use product_common::network_name::NetworkName;
pub use read_only::*;
use serde::de::DeserializeOwned;

use crate::error::{NetworkError, ObjectError};
use crate::iota_interaction_adapter::IotaClientAdapter;

/// Returns the network-id also known as chain-identifier provided by the specified iota_client
async fn network_id(iota_client: &IotaClientAdapter) -> Result<NetworkName, NetworkError> {
    let network_id = iota_client
        .read_api()
        .get_chain_identifier()
        .await
        .map_err(|e| NetworkError::RpcFailed { source: Box::new(e) })?;
    Ok(network_id.try_into().expect("chain ID is a valid network name"))
}

/// Get an object by its ID and deserialize it using BCS.
///
/// This function is used to retrieve an object from the IOTA network and deserialize it using BCS.
pub async fn get_object_ref_by_id_with_bcs<T: DeserializeOwned>(
    client: &impl CoreClientReadOnly,
    object_id: &ObjectID,
) -> Result<T, ObjectError> {
    let notarization = client
        .client_adapter()
        .read_api()
        .get_object_with_options(*object_id, IotaObjectDataOptions::bcs_lossless())
        .await
        .map_err(|err| ObjectError::RetrievalFailed {
            source: Box::new(NetworkError::RpcFailed { source: Box::new(err) }),
        })?
        .data
        .ok_or_else(|| ObjectError::NotFound {
            id: object_id.to_string(),
        })?
        .bcs
        .ok_or_else(|| ObjectError::NotFound {
            id: object_id.to_string(),
        })?
        .try_into_move()
        .ok_or_else(|| ObjectError::WrongType {
            expected: "Move object".to_string(),
            actual: "other".to_string(),
        })?
        .deserialize()
        .map_err(|err| ObjectError::RetrievalFailed { source: err.into() })?;

    Ok(notarization)
}
