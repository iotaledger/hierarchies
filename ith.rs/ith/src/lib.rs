//! # IOTA ITH (IOTA Trust Hierarchies)
//!
//!
//!
//!
//!

//! The **ITH (IOTA Trust Hierarchies)** is a non-opinionated solution designed
//! to facilitate the hierarchical distribution of trust across entities in the
//! IOTA network. It aims to simplify the process of building decentralized applications
//! on the IOTA network by providing a way to establish an additional
//! layer of trust and logic among entities.
//!
//! In ITH, a **Federation** acts as the root authority for specific properties.
//! The **Federation Owner** can delegate (accredit) trust to other entities,
//! allowing them to attest to certain properties on behalf of the root authority.
//! This creates a structured, decentralized system of trust.

pub mod client;
pub(crate) mod federation;
pub mod key;
mod provider;
pub mod types;
mod utils;
