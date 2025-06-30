use std::string::String;

use iota_interaction::MoveType;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::Argument;
use iota_sdk::types::TypeTag;
use move_core_types::ident_str;
use serde::{Deserialize, Serialize};

/// StatementValueCondition is a condition that can be applied to a StatementValue.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatementValueCondition {
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    GreaterThan(u64),
    LowerThan(u64),
}
