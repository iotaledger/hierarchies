// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use hierarchies::core::types::Accreditation;
use product_common::bindings::WasmObjectID;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::wasm_types::WasmProperty;

/// Represents an accreditation, which is a collection of properties granted by an accreditor.
#[wasm_bindgen(js_name = Accreditation, inspectable)]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct WasmAccreditation(pub(crate) Accreditation);

#[wasm_bindgen(js_class = Accreditation)]
impl WasmAccreditation {
    /// Returns the unique identifier of the accreditation.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> WasmObjectID {
        self.0.id.object_id().to_string()
    }

    /// Returns the identifier of the entity that granted the accreditation.
    #[wasm_bindgen(getter, js_name = "accreditedBy")]
    pub fn accredited_by(&self) -> String {
        self.0.accredited_by.clone()
    }

    /// Returns the properties associated with this accreditation as a map.
    #[wasm_bindgen(getter)]
    pub fn properties(&self) -> Box<[WasmProperty]> {
        self.0
            .properties
            .values()
            .map(|property| WasmProperty::from(property.clone()))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

impl From<Accreditation> for WasmAccreditation {
    fn from(value: Accreditation) -> Self {
        WasmAccreditation(value)
    }
}

impl From<WasmAccreditation> for Accreditation {
    fn from(value: WasmAccreditation) -> Self {
        value.0
    }
}
