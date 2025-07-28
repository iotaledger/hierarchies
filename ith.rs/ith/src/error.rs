// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Error types for the ITH library
//!
//! This module re-exports all domain-specific error types used throughout the library,
//! providing a single location for users to discover and import error types.
//!
//! ## Error Architecture
//!
//! The ITH library uses domain-specific error types instead of a monolithic error enum.
//! Each domain has its own error type that provides detailed context for that specific area:
//!
//! ### Common Errors
//! - [`NetworkError`] - Network and RPC related errors
//! - [`ConfigError`] - Configuration and setup errors
//! - [`ObjectError`] - Object retrieval and manipulation errors
//!
//! ### Core Operation Errors
//! - [`OperationError`] - Composite error for ITH operations
//! - [`CapabilityError`] - Capability verification and management
//!
//! ### Client Errors
//! - [`ClientError`] - Full client operations (read/write)
//!
//! ### Transaction Errors
//! - [`TransactionError`] - Transaction building and execution

use iota_interaction_rust::AdapterError;
use thiserror::Error;

// Client errors
pub use crate::client::ClientError;
// Transaction errors
pub use crate::core::transactions::TransactionError;
// Core operation errors
pub use crate::core::{CapabilityError, OperationError};

// == Common errors ==

/// Network-related errors that can occur during RPC operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum NetworkError {
    /// RPC call failed
    #[error("RPC call failed")]
    RpcFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Configuration-related errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// Package not found for the specified network
    #[error("package not found for network: {network}")]
    PackageNotFound { network: String },

    /// Invalid configuration field
    #[error("invalid configuration: {field}")]
    Invalid { field: String },
}

/// Object lookup and retrieval errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ObjectError {
    /// Object not found on the network
    #[error("object not found: {id}")]
    NotFound { id: String },

    /// Failed to retrieve object with options
    #[error("failed to retrieve object")]
    RetrievalFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Object has wrong type
    #[error("wrong object type: expected {expected}, got {actual}")]
    WrongType { expected: String, actual: String },
}

// Convert AdapterError to NetworkError
impl From<AdapterError> for NetworkError {
    fn from(err: crate::iota_interaction_adapter::AdapterError) -> Self {
        NetworkError::RpcFailed { source: Box::new(err) }
    }
}
