// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Error types for transaction operations

use thiserror::Error;

use crate::core::OperationError;

/// Errors that can occur during transaction building and execution
#[derive(Debug, Error, strum::IntoStaticStr)]
#[non_exhaustive]
pub enum TransactionError {
    /// Transaction execution failed
    #[error("transaction execution failed: {reason}")]
    ExecutionFailed { reason: String },

    /// Invalid transaction response
    #[error("invalid transaction response")]
    InvalidResponse,

    /// Transaction event processing failed
    #[error("event processing failed for event type: {event_type}")]
    EventProcessingFailed { event_type: String },

    /// Operation error during transaction
    #[error("operation error during transaction")]
    Operation(#[from] OperationError),
}
