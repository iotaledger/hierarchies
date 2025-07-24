// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # IOTA Trust Hierarchies (ITH)
//!
//! The **IOTA Trust Hierarchies (ITH)** is a non-opinionated solution designed
//! to facilitate the hierarchical distribution of trust across entities in the
//! IOTA network. It aims to simplify the process of building decentralized
//! applications on the IOTA network by providing a way to establish an
//! additional layer of trust and logic among entities.
//!
//! In ITH, a **Federation** acts as the root authority for specific properties.
//! The **Federation Owner** can delegate (accredit) trust to other entities,
//! allowing them to attest to certain properties on behalf of the root authority.
//! This creates a structured, decentralized system of trust.
//!
//! More information about ITH can be found in the [ITH documentation](https://github.com/iotaledger/ith).

pub mod client;
pub mod core;
pub mod error;
mod iota_interaction_adapter;
pub mod package;
mod utils;

#[cfg(feature = "gas-station")]
pub mod http_client {
    pub use product_common::http_client::*;
}
