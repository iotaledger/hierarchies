use std::str::FromStr;

use iota_interaction::ident_str;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::Argument;
use iota_interaction::types::{TypeTag, MOVE_STDLIB_PACKAGE_ID};
use iota_interaction::MoveType;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default, Deserialize)]
pub struct Timespan {
    pub valid_from_ms: Option<u64>,
    pub valid_until_ms: Option<u64>,
}

impl Timespan {
    pub fn into_ptb(self, ptb: &mut ProgrammableTransactionBuilder, package_id: ObjectID) -> Result<Argument, Error> {
        todo!()
    }
}

impl MoveType for Timespan {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{}::statement::Timespan", package).as_str()).expect("Failed to create type tag")
    }
}
