// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use hierarchies::core::types::property_name::PropertyName;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = PropertyName, inspectable)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct WasmPropertyName(pub(crate) PropertyName);

#[wasm_bindgen(js_class = PropertyName)]
impl WasmPropertyName {
    /// Creates a new `WasmPropertyName` from a `js_sys::Array` of strings.
    ///
    /// # Arguments
    ///
    /// * `names` - The string representations of the property name.
    #[wasm_bindgen(constructor)]
    pub fn new(names: js_sys::Array) -> Self {
        let names = names.iter().map(|v| v.as_string().unwrap()).collect::<Vec<_>>();
        Self(PropertyName::new(names))
    }

    /// Returns the property names as
    #[wasm_bindgen(js_name = getNames, unchecked_return_type = "Array<String>")]
    pub fn get_names(&self) -> js_sys::Array {
        self.0.names().iter().map(JsValue::from).collect()
    }

    /// Returns the dotted representation of the property name.
    #[wasm_bindgen(js_name = dotted)]
    pub fn dotted(&self) -> String {
        self.0.names().join(".").to_string()
    }
}

impl From<PropertyName> for WasmPropertyName {
    fn from(value: PropertyName) -> Self {
        WasmPropertyName(value)
    }
}

impl From<WasmPropertyName> for PropertyName {
    fn from(value: WasmPropertyName) -> Self {
        value.0
    }
}
