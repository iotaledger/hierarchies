// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Iota Interaction Adapter
//!
//! This module provides a platform compile switch to provide the correct
//! adapter types from `iota_interaction_rust` or `iota_interaction_ts`.
//!
//! The adapter types are used to interact with the IOTA network.

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use iota_interaction_rust::*;
#[cfg(target_arch = "wasm32")]
pub(crate) use iota_interaction_ts::*;
