// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Hierarchies Statement Name
//!
//! This module provides a struct for representing a statement name.

use std::str::FromStr;

use iota_interaction::types::TypeTag;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::Argument;
use iota_interaction::{MoveType, ident_str};
use serde::{Deserialize, Serialize};

/// PropertyName represents the name of a Property
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PropertyName {
    names: Vec<String>,
}

impl<D> From<D> for PropertyName
where
    D: Into<String>,
{
    fn from(name: D) -> Self {
        Self {
            names: vec![name.into()],
        }
    }
}

impl PropertyName {
    /// Create a new PropertyName
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
        new_property_name(self, ptb, package_id)
    }
}

impl MoveType for PropertyName {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{package}::property_name::PropertyName").as_str())
            .expect("Failed to create type tag")
    }
}

/// Creates a new move type for a Property name
pub(crate) fn new_property_name(
    name: &PropertyName,
    ptb: &mut ProgrammableTransactionBuilder,
    package_id: ObjectID,
) -> anyhow::Result<Argument> {
    let names = ptb.pure(name.names())?;
    let property_names: Argument = ptb.programmable_move_call(
        package_id,
        ident_str!("property_name").into(),
        ident_str!("new_property_name_from_vector").into(),
        vec![],
        vec![names],
    );

    Ok(property_names)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_trusted_statement_name() {
        let name = PropertyName::new(["name", "name2"]);

        let json = json!({
          "names": ["name", "name2"]
        });

        assert_eq!(serde_json::to_value(&name).unwrap(), json);
        assert_eq!(serde_json::from_value::<PropertyName>(json).unwrap(), name);
    }
}
