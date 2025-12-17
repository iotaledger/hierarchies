// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Transaction Modules
//!
//! This module contains all transaction implementations for the Hierarchies system.
//! Each transaction module provides a structured way to build and execute
//! specific operations on the Hierarchies blockchain.

pub mod add_root_authority;
pub mod error;
mod new_federation;
pub mod permissions;
pub mod properties;
pub mod reinstate_root_authority;
pub mod revoke_root_authority;
pub mod validate_property;

// Re-export error types
pub use add_root_authority::*;
pub use error::TransactionError;
pub use new_federation::*;
pub use permissions::*;
pub use reinstate_root_authority::*;
pub use revoke_root_authority::*;
pub use validate_property::*;
