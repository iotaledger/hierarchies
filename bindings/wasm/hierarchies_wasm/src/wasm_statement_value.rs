// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use ith::core::types::StatementValue;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// use crate::wasm_time_lock::WasmTimeLock;

#[wasm_bindgen(js_name = StatementValue, inspectable)]
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

// #[wasm_bindgen(js_class = Data)]

// impl WasmData {}

// /// Represents the different types of data that can be notarized.
// #[wasm_bindgen(js_name = Data, inspectable)]
// pub struct WasmData(pub(crate) Data);

// #[wasm_bindgen(js_class = Data)]
// impl WasmData {
//     /// Retrieves the value of the data as a `any`.
//     ///
//     /// # Returns
//     /// A `any` containing the data, either as bytes or text.
//     #[wasm_bindgen(getter)]
//     pub fn value(&self) -> JsValue {
//         match &self.0 {
//             Data::Bytes(bytes) => JsValue::from(bytes.clone()),
//             Data::Text(text) => JsValue::from(text),
//         }
//     }

//     /// Converts the data to a string representation.
//     ///
//     /// # Returns
//     /// A `String` containing the text representation of the data.
//     #[wasm_bindgen(js_name = toString)]
//     pub fn to_string(&self) -> String {
//         match &self.0 {
//             Data::Bytes(bytes) => String::from_utf8_lossy(bytes).to_string(),
//             Data::Text(text) => text.to_string(),
//         }
//     }

//     /// Converts the data to a byte array.
//     ///
//     /// # Returns
//     /// A `Uint8Array` containing the byte representation of the data.
//     #[wasm_bindgen(js_name = toBytes)]
//     pub fn to_bytes(&self) -> Vec<u8> {
//         match &self.0 {
//             Data::Bytes(bytes) => bytes.clone(),
//             Data::Text(text) => text.clone().as_bytes().to_vec(),
//         }
//     }
// }

// impl From<Data> for WasmData {
//     fn from(value: Data) -> Self {
//         WasmData(value)
//     }
// }

// impl From<WasmData> for Data {
//     fn from(value: WasmData) -> Self {
//         serde_wasm_bindgen::from_value(value.into()).expect("From implementation works")
//     }
// }

// /// Represents the state of a hierarchies.
// ///
// /// State encapsulates the data being notarized along with optional metadata.
// /// It serves as the primary content container for both locked and dynamic
// /// hierarchiess.
// #[wasm_bindgen(js_name = State, inspectable)]
// pub struct WasmState(pub(crate) State);

// #[wasm_bindgen(js_class = State)]
// impl WasmState {
//     /// Retrieves the data associated with the state.
//     ///
//     /// # Returns
//     /// A `Data` instance containing the state data.
//     #[wasm_bindgen(getter)]
//     pub fn data(&self) -> WasmData {
//         self.0.data.clone().into()
//     }

//     /// Retrieves the metadata associated with the state.
//     ///
//     /// # Returns
//     /// A `string` containing the metadata, if existing.
//     #[wasm_bindgen(getter)]
//     pub fn metadata(&self) -> Option<String> {
//         self.0.metadata.clone()
//     }

//     /// Creates a new state from a string.
//     ///
//     /// Use this for text data like documents, JSON, or configuration.
//     ///
//     /// # Arguments
//     /// * `data` - The string data for the state.
//     /// * `metadata` - Optional metadata for the state.
//     ///
//     /// # Returns
//     /// A new `State` instance.
//     #[wasm_bindgen(js_name = fromString)]
//     pub fn from_string(data: String, metadata: Option<String>) -> Self {
//         WasmState(State::from_string(data, metadata))
//     }

//     /// Creates a new state from raw bytes.
//     ///
//     /// Use this for binary data like files, images, or serialized content.
//     ///
//     /// # Arguments
//     /// * `data` - The byte array data for the state.
//     /// * `metadata` - Optional metadata for the state.
//     ///
//     /// # Returns
//     /// A new `State` instance.
//     #[wasm_bindgen(js_name = fromBytes)]
//     pub fn from_bytes(data: Uint8Array, metadata: Option<String>) -> Self {
//         WasmState(State::from_bytes(data.to_vec(), metadata))
//     }
// }

// impl From<State> for WasmState {
//     fn from(value: State) -> Self {
//         WasmState(value)
//     }
// }

// impl From<WasmState> for State {
//     fn from(value: WasmState) -> Self {
//         value.0
//     }
// }

// /// Represents the lock metadata of a hierarchies.
// #[wasm_bindgen(js_name = LockMetadata, getter_with_clone, inspectable)]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct WasmLockMetadata {
//     /// The update lock configuration.
//     #[wasm_bindgen(js_name = updateLock)]
//     pub update_lock: WasmTimeLock,
//     /// The delete lock configuration.
//     #[wasm_bindgen(js_name = deleteLock)]
//     pub delete_lock: WasmTimeLock,
//     /// The transfer lock configuration.
//     #[wasm_bindgen(js_name = transferLock)]
//     pub transfer_lock: WasmTimeLock,
// }

// impl From<LockMetadata> for WasmLockMetadata {
//     fn from(value: LockMetadata) -> Self {
//         WasmLockMetadata {
//             update_lock: WasmTimeLock(value.update_lock),
//             delete_lock: WasmTimeLock(value.delete_lock),
//             transfer_lock: WasmTimeLock(value.transfer_lock),
//         }
//     }
// }

// impl From<WasmLockMetadata> for LockMetadata {
//     fn from(value: WasmLockMetadata) -> Self {
//         serde_wasm_bindgen::from_value(value.into()).expect("From implementation works")
//     }
// }

// /// Represents immutable metadata of a hierarchies.
// #[wasm_bindgen(js_name = ImmutableMetadata, inspectable)]
// pub struct WasmImmutableMetadata(pub(crate) ImmutableMetadata);

// #[wasm_bindgen(js_class = ImmutableMetadata)]
// impl WasmImmutableMetadata {
//     /// Retrieves the timestamp when the hierarchies was created.
//     ///
//     /// # Returns
//     /// The timestamp as `number` value representing the seconds since the Unix epoch.
//     #[wasm_bindgen(js_name = createdAt, getter)]
//     pub fn created_at(&self) -> u64 {
//         self.0.created_at
//     }

//     /// Retrieves the description of the hierarchies.
//     ///
//     /// # Returns
//     /// A description `string`, if existing.
//     #[wasm_bindgen(getter)]
//     pub fn description(&self) -> Option<String> {
//         self.0.description.clone()
//     }

//     /// Retrieves the optional lock metadata for the hierarchies.
//     ///
//     /// # Returns
//     /// A `LockMetadata` instance, if existing.
//     #[wasm_bindgen(getter)]
//     pub fn locking(&self) -> Option<WasmLockMetadata> {
//         self.0.locking.clone().map(|l| l.into())
//     }
// }

// /// Represents the hierarchies method of a hierarchies object.
// ///
// /// This enum defines the possible methods for a hierarchies:
// /// - `Dynamic`: Dynamic hierarchies.
// /// - `Locked`: Locked hierarchies.
// #[wasm_bindgen(js_name = HierarchiesMethod)]
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum WasmHierarchiesMethod {
//     Dynamic = "Dynamic",
//     Locked = "Locked",
// }

// impl From<HierarchiesMethod> for WasmHierarchiesMethod {
//     fn from(value: HierarchiesMethod) -> Self {
//         match value {
//             HierarchiesMethod::Dynamic => WasmHierarchiesMethod::Dynamic,
//             HierarchiesMethod::Locked => WasmHierarchiesMethod::Locked,
//         }
//     }
// }

// impl From<WasmHierarchiesMethod> for HierarchiesMethod {
//     fn from(value: WasmHierarchiesMethod) -> Self {
//         match value {
//             WasmHierarchiesMethod::Dynamic => HierarchiesMethod::Dynamic,
//             WasmHierarchiesMethod::Locked => HierarchiesMethod::Locked,
//             WasmHierarchiesMethod::__Invalid => {
//                 panic!("The HierarchiesMethod {value:?} is not known")
//             }
//         }
//     }
// }
