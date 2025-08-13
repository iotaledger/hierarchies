// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Capabilities
//!
//! This module provides capability types for the Hierarchies (IOTA Trust Hierarchy)
//! module.

use core::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Capabilities are the different types of capabilities that can be issued
/// to an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    RootAuthority,
    Accredit,
}

impl Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Capability::RootAuthority => write!(f, "RootAuthorityCap"),
            Capability::Accredit => write!(f, "AccreditCap"),
        }
    }
}
