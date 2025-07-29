// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Domain-specific error types for ITH core operations

use thiserror::Error;

use crate::error::ObjectError;

/// Errors that can occur during ITH operations
#[derive(Debug, Error, strum::IntoStaticStr)]
#[non_exhaustive]
pub enum OperationError {
    /// Capability operation failed
    #[error("capability operation failed")]
    Capability(#[from] CapabilityError),

    /// Object operation failed
    #[error("object operation failed")]
    Object(#[from] ObjectError),

    /// BCS serialization failed
    #[error("serialization failed")]
    Serialization {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Any error
    #[error("any error")]
    Any {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl From<bcs::Error> for OperationError {
    fn from(err: bcs::Error) -> Self {
        OperationError::Serialization { source: Box::new(err) }
    }
}

impl From<anyhow::Error> for OperationError {
    fn from(err: anyhow::Error) -> Self {
        OperationError::Any {
            source: err.into_boxed_dyn_error(),
        }
    }
}

/// Errors that can occur during capability operations
#[derive(Debug, Error, strum::IntoStaticStr)]
#[non_exhaustive]
pub enum CapabilityError {
    /// Capability not found
    #[error("capability '{cap_type}' not found for owner")]
    NotFound { cap_type: String },

    /// Invalid capability type
    #[error("invalid capability type: {cap_type}")]
    InvalidType { cap_type: String },
}
