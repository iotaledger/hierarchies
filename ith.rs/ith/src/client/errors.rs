// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Error types for ITH client operations

use thiserror::Error;

use crate::core::transactions::TransactionError;
use crate::core::{AccreditationError, CapabilityError, FederationError, OperationError, StatementError};
use crate::error::{ConfigError, NetworkError, ObjectError};

/// High-level error type for ITH client operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ClientError {
    /// Federation operation failed
    #[error(transparent)]
    Federation(#[from] FederationError),

    /// Statement operation failed
    #[error(transparent)]
    Statement(#[from] StatementError),

    /// Capability operation failed
    #[error(transparent)]
    Capability(#[from] CapabilityError),

    /// Accreditation operation failed
    #[error(transparent)]
    Accreditation(#[from] AccreditationError),

    /// Transaction operation failed
    #[error(transparent)]
    Transaction(#[from] TransactionError),

    /// Network operation failed
    #[error(transparent)]
    Network(#[from] NetworkError),

    /// Configuration error
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Client not initialized properly
    #[error("client not initialized: {reason}")]
    NotInitialized { reason: String },

    /// Invalid input provided to client method
    #[error("invalid input: {details}")]
    InvalidInput { details: String },
}

/// Errors specific to read-only client operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ReadOnlyClientError {
    /// Network error
    #[error(transparent)]
    Network(#[from] NetworkError),

    /// Configuration error
    #[error(transparent)]
    Configuration(#[from] ConfigError),

    /// Execution failed
    #[error("execution failed: {reason}")]
    ExecutionFailed { reason: String },

    /// Invalid response from network
    #[error("invalid response: {reason}")]
    InvalidResponse { reason: String },

    /// Object error
    #[error(transparent)]
    Object(#[from] ObjectError),

    /// Operation error
    #[error(transparent)]
    Operation(#[from] OperationError),
}
