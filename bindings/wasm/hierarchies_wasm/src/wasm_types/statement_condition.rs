// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ith::core::types::statements::condition::StatementValueCondition;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = StatementCondition, inspectable)]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WasmStatementCondition(pub(crate) StatementValueCondition);

#[wasm_bindgen(js_class = StatementCondition)]
impl WasmStatementCondition {
    /// Creates a new `StatementValueCondition` of type `StartsWith`.
    #[wasm_bindgen(js_name = newStartsWith)]
    pub fn new_starts_with(text: String) -> Self {
        Self(StatementValueCondition::StartsWith(text))
    }

    /// Creates a new `StatementValueCondition` of type `EndsWith`.
    #[wasm_bindgen(js_name = newEndsWith)]
    pub fn new_ends_with(text: String) -> Self {
        Self(StatementValueCondition::EndsWith(text))
    }

    /// Creates a new `StatementValueCondition` of type `Contains`.
    #[wasm_bindgen(js_name = newContains)]
    pub fn new_contains(text: String) -> Self {
        Self(StatementValueCondition::Contains(text))
    }

    /// Creates a new `StatementValueCondition` of type `GreaterThan`.
    #[wasm_bindgen(js_name = newGreaterThan)]
    pub fn new_greater_than(value: u64) -> Self {
        Self(StatementValueCondition::GreaterThan(value))
    }

    /// Creates a new `StatementValueCondition` of type `LowerThan`.
    #[wasm_bindgen(js_name = newLowerThan)]
    pub fn new_lower_than(value: u64) -> Self {
        Self(StatementValueCondition::LowerThan(value))
    }

    /// Returns `true` if the `StatementValueCondition` is of type `StartsWith`.
    #[wasm_bindgen(js_name = isStartsWith)]
    pub fn is_starts_with(&self) -> bool {
        matches!(self.0, StatementValueCondition::StartsWith(_))
    }

    /// Returns `true` if the `StatementValueCondition` is of type `EndsWith`.
    #[wasm_bindgen(js_name = isEndsWith)]
    pub fn is_ends_with(&self) -> bool {
        matches!(self.0, StatementValueCondition::EndsWith(_))
    }

    /// Returns `true` if the `StatementValueCondition` is of type `Contains`.
    #[wasm_bindgen(js_name = isContains)]
    pub fn is_contains(&self) -> bool {
        matches!(self.0, StatementValueCondition::Contains(_))
    }

    /// Returns `true` if the `StatementValueCondition` is of type `GreaterThan`.
    #[wasm_bindgen(js_name = isGreaterThan)]
    pub fn is_greater_than(&self) -> bool {
        matches!(self.0, StatementValueCondition::GreaterThan(_))
    }

    /// Returns `true` if the `StatementValueCondition` is of type `LowerThan`.
    #[wasm_bindgen(js_name = isLowerThan)]
    pub fn is_lower_than(&self) -> bool {
        matches!(self.0, StatementValueCondition::LowerThan(_))
    }

    /// Returns the `String` value if the `StatementValueCondition` is of type `StartsWith`.
    #[wasm_bindgen(js_name = asStartsWith)]
    pub fn as_starts_with(&self) -> Option<String> {
        if let StatementValueCondition::StartsWith(text) = &self.0 {
            Some(text.clone())
        } else {
            None
        }
    }

    /// Returns the `String` value if the `StatementValueCondition` is of type `EndsWith`.
    #[wasm_bindgen(js_name = asEndsWith)]
    pub fn as_ends_with(&self) -> Option<String> {
        if let StatementValueCondition::EndsWith(text) = &self.0 {
            Some(text.clone())
        } else {
            None
        }
    }

    /// Returns the `String` value if the `StatementValueCondition` is of type `Contains`.
    #[wasm_bindgen(js_name = asContains)]
    pub fn as_contains(&self) -> Option<String> {
        if let StatementValueCondition::Contains(text) = &self.0 {
            Some(text.clone())
        } else {
            None
        }
    }

    /// Returns the `u64` value if the `StatementValueCondition` is of type `GreaterThan`.
    #[wasm_bindgen(js_name = asGreaterThan)]
    pub fn as_greater_than(&self) -> Option<u64> {
        if let StatementValueCondition::GreaterThan(value) = self.0 {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the `u64` value if the `StatementValueCondition` is of type `LowerThan`.
    #[wasm_bindgen(js_name = asLowerThan)]
    pub fn as_lower_than(&self) -> Option<u64> {
        if let StatementValueCondition::LowerThan(value) = self.0 {
            Some(value)
        } else {
            None
        }
    }
}

impl From<StatementValueCondition> for WasmStatementCondition {
    fn from(value: StatementValueCondition) -> Self {
        WasmStatementCondition(value)
    }
}

impl From<WasmStatementCondition> for StatementValueCondition {
    fn from(value: WasmStatementCondition) -> Self {
        value.0
    }
}
