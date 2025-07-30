// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ith::core::types::statements::value::StatementValue;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// use crate::wasm_time_lock::WasmTimeLock;

#[wasm_bindgen(js_name = StatementValue, inspectable)]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WasmStatementValue(pub(crate) StatementValue);

#[wasm_bindgen(js_class = StatementValue)]
impl WasmStatementValue {
    /// Creates a new `StatementValue` of type `Text`.
    ///
    /// # Arguments
    ///
    /// * `text` - The string value.
    #[wasm_bindgen(js_name = fromText)]
    pub fn from_text(text: String) -> Self {
        Self(StatementValue::Text(text))
    }

    /// Creates a new `StatementValue` of type `Number`.
    ///
    /// # Arguments
    ///
    /// * `number` - The numeric value.
    #[wasm_bindgen(js_name = fromNumber)]
    pub fn from_number(number: u64) -> Self {
        Self(StatementValue::Number(number))
    }

    /// Returns `true` if the `StatementValue` is of type `Text`.
    #[wasm_bindgen(js_name = isText)]
    pub fn is_text(&self) -> bool {
        matches!(self.0, StatementValue::Text(_))
    }

    /// Returns `true` if the `StatementValue` is of type `Number`.
    #[wasm_bindgen(js_name = isNumber)]
    pub fn is_number(&self) -> bool {
        matches!(self.0, StatementValue::Number(_))
    }

    /// Returns the `String` value if the `StatementValue` is of type `Text`.
    ///
    /// # Returns
    ///
    /// The string value, or `undefined` if the type is not `Text`.
    #[wasm_bindgen(js_name = asText)]
    pub fn as_text(&self) -> Option<String> {
        if let StatementValue::Text(text) = &self.0 {
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
        if let StatementValue::Number(number) = self.0 {
            Some(number)
        } else {
            None
        }
    }
}

impl From<StatementValue> for WasmStatementValue {
    fn from(value: StatementValue) -> Self {
        WasmStatementValue(value)
    }
}

impl From<WasmStatementValue> for StatementValue {
    fn from(value: WasmStatementValue) -> Self {
        value.0
    }
}
