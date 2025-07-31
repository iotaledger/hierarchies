// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Statement Value Condition
//!
//! This module provides a condition that can be applied to a StatementValue.

use std::str::FromStr;
use std::string::String;

use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::Argument;
use iota_interaction::types::TypeTag;
use iota_interaction::{ident_str, MoveType};
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

impl StatementValueCondition {
    pub fn into_ptb(self, ptb: &mut ProgrammableTransactionBuilder, package_id: ObjectID) -> anyhow::Result<Argument> {
        match self {
            StatementValueCondition::StartsWith(text) => new_condition_starts_with(text, ptb, package_id),
            StatementValueCondition::EndsWith(text) => new_condition_ends_with(text, ptb, package_id),
            StatementValueCondition::Contains(text) => new_condition_contains(text, ptb, package_id),
            StatementValueCondition::GreaterThan(value) => new_condition_greater_than(value, ptb, package_id),
            StatementValueCondition::LowerThan(value) => new_condition_lower_than(value, ptb, package_id),
        }
    }
}

impl MoveType for StatementValueCondition {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{package}::statement_condition::StatementValueCondition").as_str())
            .expect("Failed to create type tag")
    }
}

/// Creates a new move type for a Statement name
pub(crate) fn new_condition_starts_with(
    text: String,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let names = ptb.pure(text)?;
    let condition: Argument = ptb.programmable_move_call(
        package_id,
        ident_str!("statement_condition").into(),
        ident_str!("new_condition_starts_with").into(),
        vec![],
        vec![names],
    );

    Ok(condition)
}

fn new_condition_ends_with(
    text: String,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let names = ptb.pure(text)?;
    let condition: Argument = ptb.programmable_move_call(
        package_id,
        ident_str!("statement_condition").into(),
        ident_str!("new_condition_ends_with").into(),
        vec![],
        vec![names],
    );

    Ok(condition)
}

fn new_condition_contains(
    text: String,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let names = ptb.pure(text)?;
    let condition: Argument = ptb.programmable_move_call(
        package_id,
        ident_str!("statement_condition").into(),
        ident_str!("new_condition_contains").into(),
        vec![],
        vec![names],
    );

    Ok(condition)
}

fn new_condition_greater_than(
    value: u64,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let names = ptb.pure(value)?;
    let condition: Argument = ptb.programmable_move_call(
        package_id,
        ident_str!("statement_condition").into(),
        ident_str!("new_condition_greater_than").into(),
        vec![],
        vec![names],
    );
    Ok(condition)
}

fn new_condition_lower_than(
    value: u64,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let names = ptb.pure(value)?;
    let condition: Argument = ptb.programmable_move_call(
        package_id,
        ident_str!("statement_condition").into(),
        ident_str!("new_condition_lower_than").into(),
        vec![],
        vec![names],
    );
    Ok(condition)
}
