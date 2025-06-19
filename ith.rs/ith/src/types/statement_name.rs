use std::str::FromStr;

use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_sdk::types::transaction::Argument;
use iota_sdk::types::TypeTag;
use move_core_types::ident_str;
use serde::{Deserialize, Serialize};

use crate::utils::MoveType;

/// StatementName represents the name of a Statement
