// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_interaction::types::TypeTag;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::Argument;
use iota_interaction::{MoveType, ident_str};
use serde::{Deserialize, Serialize};

/// PropertyValue represents the value of a Property
/// It can be either a text or a number
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum PropertyValue {
    Text(String),
    Number(u64),
}

impl PropertyValue {
    /// Converts the PropertyValue to a ProgrammableTransactionBuilder argument
    pub(crate) fn to_ptb(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        package_id: ObjectID,
    ) -> anyhow::Result<Argument> {
        match self.clone() {
            PropertyValue::Text(text) => new_property_value_string(text, ptb, package_id),
            PropertyValue::Number(number) => new_property_value_number(number, ptb, package_id),
        }
    }
}

/// Creates a new move type for a Property value string
pub(crate) fn new_property_value_string(
    value: String,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let v = ptb.pure(value)?;
    Ok(ptb.programmable_move_call(
        package_id,
        ident_str!("property_value").into(),
        ident_str!("new_property_value_string").into(),
        vec![],
        vec![v],
    ))
}

/// Creates a new move type for a Property value number
pub(crate) fn new_property_value_number(
    value: u64,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let v = ptb.pure(value)?;
    Ok(ptb.programmable_move_call(
        package_id,
        ident_str!("property_value").into(),
        ident_str!("new_property_value_number").into(),
        vec![],
        vec![v],
    ))
}

impl MoveType for PropertyValue {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{package}::property_value::PropertyValue").as_str())
            .expect("Failed to create type tag")
    }
}
