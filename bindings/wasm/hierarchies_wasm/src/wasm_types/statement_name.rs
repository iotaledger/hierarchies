// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ith::core::types::statements::name::StatementName;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = StatementName, inspectable)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WasmStatementName(pub(crate) StatementName);

#[wasm_bindgen(js_class = StatementName)]
impl WasmStatementName {
    /// Creates a new `WasmStatementName` from a `js_sys::Array` of strings.
    ///
    /// # Arguments
    ///
    /// * `names` - The string representations of the statement name.
    #[wasm_bindgen(constructor)]
    pub fn new(names: js_sys::Array) -> Self {
        let names = names.iter().map(|v| v.as_string().unwrap()).collect::<Vec<_>>();
        Self(StatementName::new(names))
    }

    /// Returns the statement names as
    #[wasm_bindgen(js_name = getNames, unchecked_return_type = "Array<String>")]
    pub fn get_names(&self) -> js_sys::Array {
        self.0.names().iter().map(JsValue::from).collect()
    }

    /// Returns the dotted representation of the statement name.
    #[wasm_bindgen(js_name = dotted)]
    pub fn dotted(&self) -> String {
        self.0.names().join(".").to_string()
    }
}

impl From<StatementName> for WasmStatementName {
    fn from(value: StatementName) -> Self {
        WasmStatementName(value)
    }
}

impl From<WasmStatementName> for StatementName {
    fn from(value: WasmStatementName) -> Self {
        value.0
    }
}
