// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use hierarchies::core::types::value::PropertyValue;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// use crate::wasm_time_lock::WasmTimeLock;

#[wasm_bindgen(js_name = PropertyValue, inspectable)]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WasmPropertyValue(pub(crate) PropertyValue);

#[wasm_bindgen(js_class = PropertyValue)]
impl WasmPropertyValue {
    /// Creates a new `PropertyValue` of type `Text`.
    ///
    /// # Arguments
    ///
    /// * `text` - The string value.
    #[wasm_bindgen(js_name = newText)]
    pub fn new_text(text: String) -> Self {
        Self(PropertyValue::Text(text))
    }

    /// Creates a new `StatementValue` of type `Number`.
    ///
    /// # Arguments
    ///
    /// * `number` - The numeric value.
    #[wasm_bindgen(js_name = newNumber)]
    pub fn new_number(number: u64) -> Self {
        Self(PropertyValue::Number(number))
    }

    /// Returns `true` if the `StatementValue` is of type `Text`.
    #[wasm_bindgen(js_name = isText)]
    pub fn is_text(&self) -> bool {
        matches!(self.0, PropertyValue::Text(_))
    }

    /// Returns `true` if the `StatementValue` is of type `Number`.
    #[wasm_bindgen(js_name = isNumber)]
    pub fn is_number(&self) -> bool {
        matches!(self.0, PropertyValue::Number(_))
    }

    /// Returns the `String` value if the `StatementValue` is of type `Text`.
    ///
    /// # Returns
    ///
    /// The string value, or `undefined` if the type is not `Text`.
    #[wasm_bindgen(js_name = asText)]
    pub fn as_text(&self) -> Option<String> {
        if let PropertyValue::Text(text) = &self.0 {
            Some(text.clone())
        } else {
            None
        }
    }

    /// Returns the `u64` value if the `StatementValue` is of type `Number`.
    ///
    /// # Returns
    ///
    /// The numeric value, or `undefined` if the type is not `Number`.
    #[wasm_bindgen(js_name = asNumber)]
    pub fn as_number(&self) -> Option<u64> {
        if let PropertyValue::Number(number) = self.0 {
            Some(number)
        } else {
            None
        }
    }
}

impl From<PropertyValue> for WasmPropertyValue {
    fn from(value: PropertyValue) -> Self {
        WasmPropertyValue(value)
    }
}

impl From<WasmPropertyValue> for PropertyValue {
    fn from(value: WasmPropertyValue) -> Self {
        value.0
    }
}
