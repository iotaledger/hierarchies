// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ith::core::types::statements::name::StatementName;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = StatementName, inspectable)]
#[derive(Deserialize, Serialize)]
pub struct WasmStatementName(pub(crate) StatementName);

impl From<StatementName> for WasmStatementName {
    fn from(value: StatementName) -> Self {
        WasmStatementName(value)
    }
}

impl From<WasmStatementName> for StatementName {
    fn from(value: WasmStatementName) -> Self {
        serde_wasm_bindgen::from_value(value.into()).expect("From implementation works")
    }
}
