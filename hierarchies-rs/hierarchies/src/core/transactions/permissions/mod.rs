// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Permissions
//!
//! This module provides the transactions for managing permissions in the Hierarchies system.
//!
//! ## Permissions
//!
//! - `create_accreditation_to_accredit`: Create accreditation to accredit
//! - `create_accreditation_to_attest`: Create accreditation to attest
//! - `revoke_accreditation_to_accredit`: Revoke accreditation to accredit
//! - `revoke_accreditation_to_attest`: Revoke accreditation to attest
//!
//! ## Transactions
//!
//! - `CreateAccreditationToAccredit`: Create accreditation to accredit
//! - `CreateAccreditationToAttest`: Create accreditation to attest
//! - `RevokeAccreditationToAccredit`: Revoke accreditation to accredit
//! - `RevokeAccreditationToAttest`: Revoke accreditation to attest

mod create_accreditation_to_accredit;
mod create_accreditation_to_attest;
mod revoke_accreditation_to_accredit;
mod revoke_accreditation_to_attest;

pub use create_accreditation_to_accredit::*;
pub use create_accreditation_to_attest::*;
pub use revoke_accreditation_to_accredit::*;
pub use revoke_accreditation_to_attest::*;
