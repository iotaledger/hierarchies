use serde::{Deserialize, Serialize};

/// Capabilities are the different types of capabilities that can be issued
/// to an account
#[derive(Debug, strum::Display, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
  #[strum(serialize = "RootAuthorityCap")]
  RootAuthority,
  #[strum(serialize = "AttestCap")]
  Attest,
  #[strum(serialize = "AccreditCap")]
  Accredit,
}
