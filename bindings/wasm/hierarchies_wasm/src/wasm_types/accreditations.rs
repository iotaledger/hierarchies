// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::wasm_types::accreditation::WasmAccreditation;

/// Represents a collection of accreditation statements
#[wasm_bindgen(js_name = Accreditations, inspectable)]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct WasmAccreditations(pub(crate) ith::core::types::Accreditations);

#[wasm_bindgen(js_class = Accreditations)]
impl WasmAccreditations {
    /// Returns the accreditations as an array.
    #[wasm_bindgen(getter)]
    pub fn statements(&self) -> js_sys::Array {
        self.0
            .statements
            .iter()
            .map(|accreditation| JsValue::from(WasmAccreditation(accreditation.clone())))
            .collect()
    }
}

impl From<ith::core::types::Accreditations> for WasmAccreditations {
    fn from(value: ith::core::types::Accreditations) -> Self {
        WasmAccreditations(value)
    }
}

impl From<WasmAccreditations> for ith::core::types::Accreditations {
    fn from(value: WasmAccreditations) -> Self {
        value.0
    }
}
