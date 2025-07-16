// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ith::core::types::statements::condition::StatementValueCondition;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = StatementCondition, inspectable)]
pub struct WasmStatementCondition(pub(crate) StatementValueCondition);

impl From<StatementValueCondition> for WasmStatementCondition {
    fn from(value: StatementValueCondition) -> Self {
        WasmStatementCondition(value)
    }
}

impl From<WasmStatementCondition> for StatementValueCondition {
    fn from(value: WasmStatementCondition) -> Self {
        serde_wasm_bindgen::from_value(value.into()).expect("From implementation works")
    }
}
