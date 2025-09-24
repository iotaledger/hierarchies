// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use iota_interaction::types::TypeTag;
use iota_interaction::types::base_types::ObjectID;
use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_interaction::types::transaction::{Argument, Command};
use iota_interaction::{MoveType, ident_str};
use serde::{Deserialize, Serialize};

use crate::core::types::property_name::PropertyName;
use crate::core::types::property_shape::PropertyShape;
use crate::core::types::property_value::PropertyValue;
use crate::core::types::timespan::Timespan;
use crate::utils::{self, deserialize_vec_map, deserialize_vec_set};

// FederationProperties is a struct that contains a map of PropertyName to FederationProperty
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationProperties {
    #[serde(deserialize_with = "deserialize_vec_map")]
    pub data: HashMap<PropertyName, FederationProperty>,
}

// The evaluation order: allow_any => shape => allowed_values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationProperty {
    pub name: PropertyName,
    /// Allow only values that are in the set
    #[serde(deserialize_with = "deserialize_vec_set")]
    pub allowed_values: HashSet<PropertyValue>,
    /// Allow only values that match the shape.
    pub shape: Option<PropertyShape>,
    /// If true, the property is not applied, any value is allowed
    pub allow_any: bool,
    /// The time span of the property
    pub timespan: Timespan,
}

impl FederationProperty {
    pub fn new(name: impl Into<PropertyName>) -> Self {
        Self {
            name: name.into(),
            allowed_values: HashSet::new(),
            shape: None,
            allow_any: false,
            timespan: Timespan::default(),
        }
    }

    pub fn with_allowed_values(mut self, allowed_values: impl IntoIterator<Item = PropertyValue>) -> Self {
        self.allowed_values = allowed_values.into_iter().collect();
        self
    }

    pub fn with_expression(mut self, expression: PropertyShape) -> Self {
        self.shape = Some(expression);
        self
    }

    pub fn with_timespan(mut self, timespan: Timespan) -> Self {
        self.timespan = timespan;
        self
    }

    pub fn with_allow_any(mut self, allow_any: bool) -> Self {
        self.allow_any = allow_any;
        self
    }
}

impl MoveType for FederationProperty {
    fn move_type(package: ObjectID) -> TypeTag {
        TypeTag::from_str(format!("{package}::property::FederationProperty").as_str())
            .expect("Failed to create type tag")
    }
}

/// Creates a new move type for a Property
pub(crate) fn new_property(
    package_id: ObjectID,
    ptb: &mut ProgrammableTransactionBuilder,
    property: FederationProperty,
) -> anyhow::Result<Argument> {
    let value_tag = PropertyValue::move_type(package_id);

    let property_names = property.name.to_ptb(ptb, package_id)?;

    let allow_any = ptb.pure(property.allow_any)?;

    let allowed_values = property
        .allowed_values
        .into_iter()
        .map(|value| {
            value
                .to_ptb(ptb, package_id)
                .expect("failed to create new property value")
        })
        .collect();

    let allowed_values = utils::create_vec_set_from_move_values(allowed_values, value_tag, ptb, package_id);

    let property_shape_tag = PropertyShape::move_type(package_id);

    let shape = match property.shape {
        Some(shape) => {
            let shape_arg = shape.into_ptb(ptb, package_id)?;
            utils::option_to_move(Some(shape_arg), property_shape_tag, ptb)?
        }
        None => utils::option_to_move(None, property_shape_tag, ptb)?,
    };

    let property = ptb.programmable_move_call(
        package_id,
        ident_str!("property").into(),
        ident_str!("new_property").into(),
        vec![],
        vec![property_names, allowed_values, allow_any, shape],
    );

    Ok(property)
}

/// Creates a new move type for a list of Properties
pub(crate) fn new_properties(
    package_id: ObjectID,
    ptb: &mut ProgrammableTransactionBuilder,
    properties: Vec<FederationProperty>,
) -> anyhow::Result<Argument> {
    let mut property_args = vec![];
    for property in properties {
        let value_tag = PropertyValue::move_type(package_id);

        let property_names = property.name.to_ptb(ptb, package_id)?;

        let allow_any = ptb.pure(property.allow_any)?;

        let allowed_values = property
            .allowed_values
            .into_iter()
            .map(|value| {
                value
                    .to_ptb(ptb, package_id)
                    .expect("failed to create new property value")
            })
            .collect();

        let allowed_values = utils::create_vec_set_from_move_values(allowed_values, value_tag, ptb, package_id);

        let property_expression_tag = PropertyShape::move_type(package_id);

        let expression = match property.shape {
            Some(expression) => {
                let expression = expression.into_ptb(ptb, package_id)?;
                utils::option_to_move(Some(expression), property_expression_tag, ptb)?
            }

            None => utils::option_to_move(None, property_expression_tag, ptb)?,
        };

        let property = ptb.programmable_move_call(
            package_id,
            ident_str!("property").into(),
            ident_str!("new_property").into(),
            vec![],
            vec![property_names, allowed_values, allow_any, expression],
        );
        property_args.push(property);
    }

    Ok(ptb.command(Command::MakeMoveVec(
        Some(FederationProperty::move_type(package_id).into()),
        property_args,
    )))
}
