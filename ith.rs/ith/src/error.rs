// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::iota_interaction_adapter::AdapterError;

/// Errors that can occur when managing ITH
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Caused by invalid keys.
    #[error("invalid key: {0}")]
    InvalidKey(String),
    /// Config is invalid.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// An error caused by either a connection issue or an invalid RPC call.
    #[error("RPC error: {0}")]
    RpcError(String),
    /// The provided IOTA Client returned an error
    #[error("IOTA client error: {0}")]
    IotaClient(#[from] AdapterError),
    /// Generic error
    #[error("{0}")]
    GenericError(String),
    /// Failed to parse tag
    #[error("Failed to parse tag: {0}")]
    FailedToParseTag(String),
    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    /// Invalid unlock time
    #[error("Invalid unlock time: {0}")]
    TimeLock(String),
    /// The response from the IOTA node API was not in the expected format.
    #[error("unexpected API response: {0}")]
    UnexpectedApiResponse(String),
    /// Failed to deserialize data using BCS.
    #[error("BCS deserialization error: {0}")]
    DeserializationError(#[from] bcs::Error),
    /// The response from the IOTA node API was not in the expected format.
    #[error("unexpected API response: {0}")]
    TransactionUnexpectedResponse(String),
    /// Failed to get object with options
    #[error("Failed to get object with options: {0}")]
    ObjectLookup(String),
    /// An anyhow::error.
    #[error("Any error: {0}")]
    AnyError(#[from] anyhow::Error),
}
