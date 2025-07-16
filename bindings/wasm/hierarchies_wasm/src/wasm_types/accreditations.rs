// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Represents a collection of accreditation statements
#[wasm_bindgen(js_name = Accreditations, inspectable)]
#[derive(Deserialize, Serialize, Clone)]
pub struct WasmAccreditations(pub(crate) ith::core::types::Accreditations);

impl From<ith::core::types::Accreditations> for WasmAccreditations {
    fn from(value: ith::core::types::Accreditations) -> Self {
        WasmAccreditations(value)
    }
}

impl From<WasmAccreditations> for ith::core::types::Accreditations {
    fn from(value: WasmAccreditations) -> Self {
        serde_wasm_bindgen::from_value(value.into()).expect("From implementation works")
    }
}
