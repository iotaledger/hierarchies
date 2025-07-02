use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default, Deserialize)]
pub struct Timespan {
    pub valid_from_ms: Option<u64>,
    pub valid_until_ms: Option<u64>,
}
