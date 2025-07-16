// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # ITH Statement Name
//!
//! This module provides a struct for representing a statement name.

use std::str::FromStr;

use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::Argument;
use iota_interaction::types::TypeTag;
use iota_interaction::{ident_str, MoveType};
use serde::{Deserialize, Serialize};

/// StatementName represents the name of a Statement
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatementName {
    names: Vec<String>,
}

impl<D> From<D> for StatementName
where
    D: Into<String>,
{
    fn from(name: D) -> Self {
        Self {
            names: vec![name.into()],
        }
    }
}

impl StatementName {
    /// Create a new StatementName
    pub fn new<D>(names: impl IntoIterator<Item = D>) -> Self
    where
        D: Into<String>,
    {
        Self {
            names: names.into_iter().map(Into::into).collect(),
        }
    }

    pub fn names(&self) -> &Vec<String> {
        &self.names
    }

    pub fn to_ptb(&self, ptb: &mut ProgrammableTransactionBuilder, package_id: ObjectID) -> anyhow::Result<Argument> {
        new_statement_name(self, ptb, package_id)
    }
}

impl MoveType for StatementName {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{package}::statement_name::StatementName").as_str())
            .expect("Failed to create type tag")
    }
}

/// Creates a new move type for a Statement name
pub(crate) fn new_statement_name(
    name: &StatementName,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let names = ptb.pure(name.names())?;
    let statement_names: Argument = ptb.programmable_move_call(
        package_id,
        ident_str!("statement_name").into(),
        ident_str!("new_statement_name_from_vector").into(),
        vec![],
        vec![names],
    );

    Ok(statement_names)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_trusted_statement_name() {
        let name = StatementName::new(["name", "name2"]);

        let json = json!({
          "names": ["name", "name2"]
        });

        assert_eq!(serde_json::to_value(&name).unwrap(), json);
        assert_eq!(serde_json::from_value::<StatementName>(json).unwrap(), name);
    }
}
