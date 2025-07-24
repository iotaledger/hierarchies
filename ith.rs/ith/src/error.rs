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
//! - [`ParseError`] - Data parsing and format errors
//! - [`ObjectError`] - Object retrieval and manipulation errors
//!
//! ### Core Operation Errors
//! - [`OperationError`] - Composite error for ITH operations
//! - [`FederationError`] - Federation-specific operations
//! - [`StatementError`] - Statement management and validation
//! - [`CapabilityError`] - Capability verification and management
//! - [`AccreditationError`] - Accreditation lifecycle operations
//!
//! ### Client Errors  
//! - [`ClientError`] - Full client operations (read/write)
//! - [`ReadOnlyClientError`] - Read-only client operations
//!
//! ### Transaction Errors
//! - [`TransactionError`] - Transaction building and execution
//! - [`PermissionTransactionError`] - Permission-related transactions
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ith::error::{FederationError, ClientError};
//!
//! fn handle_federation_error(err: FederationError) {
//!     match err {
//!         FederationError::NotFound { id } => {
//!             eprintln!("Federation {id} not found");
//!         }
//!         FederationError::InsufficientPermissions { id, required } => {
//!             eprintln!("Need {required:?} capability for federation {id}");
//!         }
//!         _ => eprintln!("Federation error: {err}"),
//!     }
//! }
//! ```

use iota_interaction_rust::AdapterError;
use thiserror::Error;

// Client errors
pub use crate::client::{ClientError, ReadOnlyClientError};
// Transaction errors
pub use crate::core::transactions::TransactionError;
// Core operation errors
pub use crate::core::{AccreditationError, CapabilityError, FederationError, OperationError, StatementError};

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

/// Parsing and deserialization errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// BCS deserialization failed
    #[error("BCS deserialization failed")]
    BcsDeserializationFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
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

// Convert bcs::Error to ParseError
impl From<bcs::Error> for ParseError {
    fn from(err: bcs::Error) -> Self {
        ParseError::BcsDeserializationFailed { source: Box::new(err) }
    }
}
