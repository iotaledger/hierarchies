// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Timespan
//!
//! This module provides a struct for representing a timespan.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default, Deserialize)]
pub struct Timespan {
    pub valid_from_ms: Option<u64>,
    pub valid_until_ms: Option<u64>,
}
