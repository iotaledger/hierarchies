// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use hierarchies::core::types::property::{FederationProperties, FederationProperty};
use hierarchies::core::types::timespan::Timespan;
use hierarchies::core::types::{Federation, Governance, RootAuthority};
use product_common::bindings::WasmObjectID;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::wasm_types::{WasmAccreditations, WasmPropertyName, WasmPropertyShape, WasmPropertyValue};

/// Represents a federation. A federation is a group of entities that have agreed to work together
#[wasm_bindgen(js_name = Federation, inspectable)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WasmFederation(pub(crate) Federation);

impl From<Federation> for WasmFederation {
    fn from(value: Federation) -> Self {
        WasmFederation(value)
    }
}

#[wasm_bindgen(js_class = Federation)]
impl WasmFederation {
    /// Retrieves the ID of the federation.
    ///
    /// # Returns
    /// A string representing the federation ID.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> WasmObjectID {
        self.0.id.object_id().to_string()
    }

    /// Retrieves the governance of the federation.
    ///
    /// # Returns
    /// The governance object for the federation.
    #[wasm_bindgen(getter)]
    pub fn governance(&self) -> WasmGovernance {
        self.0.governance.clone().into()
    }

    /// Retrieves the root authorities of the federation.
    ///
    /// # Returns
    /// An array of root authorities.
    #[wasm_bindgen(getter, js_name = rootAuthorities)]
    pub fn root_authorities(&self) -> Vec<WasmRootAuthority> {
        self.0.root_authorities.iter().map(|ra| ra.clone().into()).collect()
    }

    /// Retrieves the revoked root authorities of the federation.
    ///
    /// # Returns
    /// An array of revoked root authorities.
    #[wasm_bindgen(getter, js_name = revokedRootAuthorities)]
    pub fn revoked_root_authorities(&self) -> Vec<WasmObjectID> {
        self.0
            .revoked_root_authorities
            .iter()
            .map(|ra| ra.to_string())
            .collect()
    }
}

/// Represents the governance of a federation
#[wasm_bindgen(js_name = Governance, inspectable)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WasmGovernance(pub(crate) Governance);

impl From<Governance> for WasmGovernance {
    fn from(value: Governance) -> Self {
        WasmGovernance(value)
    }
}

#[wasm_bindgen(js_class = Governance)]
impl WasmGovernance {
    /// Retrieves the ID of the governance.
    ///
    /// # Returns
    /// A string representing the governance ID.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> WasmObjectID {
        self.0.id.object_id().to_string()
    }

    /// Retrieves the properties in the governance.
    ///
    /// # Returns
    /// The properties object.
    #[wasm_bindgen(getter)]
    pub fn properties(&self) -> WasmProperties {
        self.0.properties.clone().into()
    }

    /// Retrieves the accreditations to accredit mapping.
    ///
    /// # Returns
    /// A JavaScript Map object containing accreditations to accredit.
    #[wasm_bindgen(getter, js_name = accreditationsToAccredit)]
    pub fn accreditations_to_accredit(&self) -> js_sys::Map {
        let map = js_sys::Map::new();
        for (key, value) in &self.0.accreditations_to_accredit {
            map.set(
                &wasm_bindgen::JsValue::from_str(&key.to_string()),
                &serde_wasm_bindgen::to_value(&WasmAccreditations::from(value.clone())).unwrap(),
            );
        }
        map
    }

    /// Retrieves the accreditations to attest mapping.
    ///
    /// # Returns
    /// A JavaScript Map object containing accreditations to attest.
    #[wasm_bindgen(getter, js_name = accreditationsToAttest)]
    pub fn accreditations_to_attest(&self) -> js_sys::Map {
        let map = js_sys::Map::new();
        for (key, value) in &self.0.accreditations_to_attest {
            map.set(
                &wasm_bindgen::JsValue::from_str(&key.to_string()),
                &serde_wasm_bindgen::to_value(&WasmAccreditations::from(value.clone())).unwrap(),
            );
        }
        map
    }
}

/// Represents a root authority. A root authority is an entity that has the highest level of authority in a federation
#[wasm_bindgen(js_name = RootAuthority, inspectable)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WasmRootAuthority(pub(crate) RootAuthority);

impl From<RootAuthority> for WasmRootAuthority {
    fn from(value: RootAuthority) -> Self {
        WasmRootAuthority(value)
    }
}

#[wasm_bindgen(js_class = RootAuthority)]
impl WasmRootAuthority {
    /// Retrieves the ID of the root authority.
    ///
    /// # Returns
    /// A string representing the root authority ID.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> WasmObjectID {
        self.0.id.object_id().to_string()
    }

    /// Retrieves the account ID of the root authority.
    ///
    /// # Returns
    /// A string representing the account ID.
    #[wasm_bindgen(getter, js_name = accountId)]
    pub fn account_id(&self) -> WasmObjectID {
        self.0.account_id.to_string()
    }
}

/// Properties is a struct that contains a map of PropertyName to Property
#[wasm_bindgen(js_name = Properties, inspectable)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WasmProperties(pub(crate) FederationProperties);

impl From<FederationProperties> for WasmProperties {
    fn from(value: FederationProperties) -> Self {
        WasmProperties(value)
    }
}

#[wasm_bindgen(js_class = Properties)]
impl WasmProperties {
    /// Retrieves all property names and their corresponding property data as a JavaScript Map.
    ///
    /// # Returns
    /// A list of Property objects.
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<WasmProperty> {
        self.0
            .data
            .values()
            .map(|v| WasmProperty::from(v.clone()))
            .collect::<Vec<_>>()
    }

    /// Adds a new property to the properties list
    pub fn add_property(&mut self, property: WasmProperty) {
        self.0.data.insert(property.property_name().0.clone(), property.0);
    }
}

/// Represents a property that can be granted to an account. A property
/// consists of a set of properties that must be satisfied by the account
/// in order to be granted the property.
///
/// The evaluation order: allow_any => shape => allowed_values
/// The evaluation order is determined by the possible size of the set of values
/// that match the shape.
#[wasm_bindgen(js_name = FederationProperty, inspectable)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WasmProperty(pub(crate) FederationProperty);

impl From<FederationProperty> for WasmProperty {
    fn from(value: FederationProperty) -> Self {
        WasmProperty(value)
    }
}

impl From<WasmProperty> for FederationProperty {
    fn from(value: WasmProperty) -> Self {
        value.0
    }
}

#[wasm_bindgen(js_class = FederationProperty)]
impl WasmProperty {
    #[wasm_bindgen(constructor)]
    pub fn new(property_name: &WasmPropertyName) -> Self {
        WasmProperty(FederationProperty {
            name: property_name.clone().into(),
            allowed_values: HashSet::new(),
            shape: None,
            allow_any: false,
            timespan: Timespan::default(),
        })
    }

    #[wasm_bindgen(js_name=withAllowedValues)]
    pub fn with_allowed_values(mut self, allowed_values: Vec<WasmPropertyValue>) -> Self {
        self.0.allowed_values = allowed_values.iter().cloned().map(|v| v.0).collect();
        self
    }

    #[wasm_bindgen(js_name=withCondition)]
    pub fn with_condition(mut self, condition: WasmPropertyShape) -> Self {
        self.0.shape = Some(condition.0);
        self
    }

    #[wasm_bindgen(js_name=withAllowAny)]
    pub fn with_allow_any(mut self, allow_any: bool) -> Self {
        self.0.allow_any = allow_any;
        self
    }

    /// Retrieves the property name.
    ///
    /// # Returns
    /// The property name.
    #[wasm_bindgen(getter, js_name = propertyName)]
    pub fn property_name(&self) -> WasmPropertyName {
        self.0.name.clone().into()
    }

    /// Sets the property name.
    #[wasm_bindgen(setter, js_name = propertyName)]
    pub fn set_property_name(&mut self, property_name: WasmPropertyName) {
        self.0.name = property_name.0;
    }

    /// Retrieves the allowed values for this property.
    ///
    /// # Returns
    /// An array of allowed property values.
    #[wasm_bindgen(getter, js_name = allowedValues)]
    pub fn allowed_values(&self) -> Box<[WasmPropertyValue]> {
        self.0.allowed_values.iter().map(|v| v.clone().into()).collect()
    }

    /// Sets the allowed values for this property.
    #[wasm_bindgen(setter, js_name = allowedValues)]
    pub fn set_allowed_values(&mut self, allowed_values: Vec<WasmPropertyValue>) {
        self.0.allowed_values = allowed_values.iter().cloned().map(|v| v.0).collect();
    }

    /// Retrieves the condition for this property.
    ///
    /// # Returns
    /// The property value condition if present.
    #[wasm_bindgen(getter)]
    pub fn condition(&self) -> Option<WasmPropertyShape> {
        self.0.shape.as_ref().map(|c| c.clone().into())
    }

    /// Sets the condition for this property.
    #[wasm_bindgen(setter, js_name = condition)]
    pub fn set_condition(&mut self, condition: WasmPropertyShape) {
        self.0.shape = Some(condition.0);
    }

    /// Checks if any value is allowed for this property.
    ///
    /// # Returns
    /// A boolean indicating if any value is allowed.
    #[wasm_bindgen(getter, js_name = allowAny)]
    pub fn allow_any(&self) -> bool {
        self.0.allow_any
    }

    /// Sets whether any value is allowed for this property.
    #[wasm_bindgen(setter, js_name = allowAny)]
    pub fn set_allow_any(&mut self, allow_any: bool) {
        self.0.allow_any = allow_any;
    }

    /// Retrieves the timespan for this property.
    ///
    /// # Returns
    /// The timespan object.
    #[wasm_bindgen(getter)]
    pub fn timespan(&self) -> WasmTimespan {
        self.0.timespan.clone().into()
    }

    /// Sets the timespan for this property.
    #[wasm_bindgen(setter, js_name = timespan)]
    pub fn set_timespan(&mut self, timespan: WasmTimespan) {
        self.0.timespan = timespan.0;
    }
}

/// Represents the time span of validity for a property
#[wasm_bindgen(js_name = Timespan, inspectable)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WasmTimespan(pub(crate) Timespan);

impl From<Timespan> for WasmTimespan {
    fn from(value: Timespan) -> Self {
        WasmTimespan(value)
    }
}

impl Default for WasmTimespan {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = Timespan)]
impl WasmTimespan {
    /// Creates a new `WasmTimespan` with default values.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmTimespan(Timespan::default())
    }

    /// Retrieves the start timestamp.
    ///
    /// # Returns
    /// The start timestamp in milliseconds.
    #[wasm_bindgen(getter, js_name = validFromMs)]
    pub fn valid_from_ms(&self) -> Option<u64> {
        self.0.valid_from_ms
    }

    /// Sets the start and end timestamps for the timespan.
    #[wasm_bindgen(setter, js_name = setValidFromMs)]
    pub fn set_valid_from_ms(&mut self, ms: u64) {
        self.0.valid_from_ms = Some(ms);
    }

    /// Retrieves the end timestamp.
    ///
    /// # Returns
    /// The end timestamp in milliseconds.
    #[wasm_bindgen(getter, js_name = validUntilMs)]
    pub fn valid_until_ms(&self) -> Option<u64> {
        self.0.valid_until_ms
    }

    /// Sets the end timestamp for the timespan.
    #[wasm_bindgen(setter, js_name = validUntilMs)]
    pub fn set_valid_until_ms(&mut self, ms: u64) {
        self.0.valid_until_ms = Some(ms);
    }
}
