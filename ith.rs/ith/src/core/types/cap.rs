// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # ITH Capabilities
//!
//! This module provides capability types for the ITH (IOTA Trust Hierarchy)
//! module.

use core::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Capabilities are the different types of capabilities that can be issued
/// to an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    RootAuthority,
    Attest,
    Accredit,
}

impl Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Capability::RootAuthority => write!(f, "RootAuthorityCap"),
            Capability::Attest => write!(f, "AttestCap"),
            Capability::Accredit => write!(f, "AccreditCap"),
        }
    }
}
