use iota_sdk::types::base_types::IotaAddress;
use serde::{Deserialize, Serialize};

/// An event that can be emitted by the ITH.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event<D> {
  pub data: D,
}

/// An event that is emitted when a new federation is created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationCreatedEvent {
  pub federation_address: IotaAddress,
}
