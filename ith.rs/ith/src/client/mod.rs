//! Client module provides the client interface for the ITH service.
//! Clients can be used to interact with the ITH service, create new federations,
//! add trusted properties, create attestations, and accreditations.
//!
//! There are two types of clients:
//! - Client: A client that can perform both on-chain and off-chain operations.
//!           It requires a signer with a private key. The client is represented by the [`ITHClient`] struct.
//! - ReadOnlyClient: A client that can only perform off-chain operations.
//!             It doesn't require a signer with a private key. The client is represented by the [`ITHClientReadOnly`] struct.
mod client;
mod read_only;

pub use client::*;
pub use read_only::*;
