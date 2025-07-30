// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Error types for ITH client operations

use thiserror::Error;

use crate::core::error::OperationError;
use crate::error::{ConfigError, NetworkError, ObjectError};

/// Errors specific to read-only client operations
#[derive(Debug, Error, strum::IntoStaticStr)]
#[non_exhaustive]
pub enum ClientError {
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

    /// Invalid input
    #[error("invalid input: {details}")]
    InvalidInput { details: String },
}
