// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Domain-specific error types for ITH core operations

use iota_interaction::types::base_types::ObjectID;
use thiserror::Error;

use crate::core::types::Capability;
use crate::error::{NetworkError, ObjectError};

/// Errors that can occur during ITH operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OperationError {
    /// Capability operation failed
    #[error("capability operation failed")]
    Capability(#[from] CapabilityError),

    /// Federation operation failed
    #[error("federation operation failed")]
    Federation(#[from] FederationError),

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

/// Errors that can occur during federation operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FederationError {
    /// Federation not found
    #[error("federation not found: {id}")]
    NotFound { id: ObjectID },

    /// Federation creation failed
    #[error("failed to create federation")]
    CreationFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Invalid federation state
    #[error("invalid federation state: current={current}, expected={expected}")]
    InvalidState { current: String, expected: String },

    /// Insufficient permissions to access federation
    #[error("insufficient permissions for federation {id}")]
    InsufficientPermissions { id: ObjectID, required: Capability },

    /// Network error during federation operation
    #[error("network error during federation operation")]
    Network(#[from] NetworkError),

    /// Object retrieval error
    #[error("federation object error")]
    Object(#[from] ObjectError),
}

/// Errors that can occur during statement operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StatementError {
    /// Statement not found in federation
    #[error("statement '{name}' not found in federation")]
    NotFound { name: String },

    /// Statement validation failed
    #[error("statement validation failed: {reason}")]
    ValidationFailed { reason: String },

    /// Statement has expired
    #[error("statement '{name}' expired at {expired_at}")]
    Expired { name: String, expired_at: u64 },

    /// Business rule violation
    #[error("business rule violation: {rule}")]
    BusinessRuleViolation { rule: String, details: String },

    /// Invalid statement format
    #[error("invalid statement format")]
    InvalidFormat {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Network error during statement operation
    #[error("network error during statement operation")]
    Network(#[from] NetworkError),
}

/// Errors that can occur during capability operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CapabilityError {
    /// Capability not found
    #[error("capability '{cap_type}' not found for owner")]
    NotFound { cap_type: String },

    /// Invalid capability type
    #[error("invalid capability type: {cap_type}")]
    InvalidType { cap_type: String },
}
