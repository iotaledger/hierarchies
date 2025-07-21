// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ith::core::types::statements::value::StatementValue;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// use crate::wasm_time_lock::WasmTimeLock;

#[wasm_bindgen(js_name = StatementValue, inspectable)]
#[derive(Deserialize, Serialize)]
pub struct WasmStatementValue(pub(crate) StatementValue);

impl From<StatementValue> for WasmStatementValue {
    fn from(value: StatementValue) -> Self {
        WasmStatementValue(value)
    }
}

impl From<WasmStatementValue> for StatementValue {
    fn from(value: WasmStatementValue) -> Self {
        serde_wasm_bindgen::from_value(value.into()).expect("From implementation works")
    }
}
