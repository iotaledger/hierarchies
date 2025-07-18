// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(deprecated)]
#![allow(clippy::upper_case_acronyms)]
// wasm_bindgen calls drop on non-Drop types. When/If this is fixed, this can be removed (no issue to link here yet).
#![allow(clippy::drop_non_drop)]
#![allow(clippy::unused_unit)]
// complains about RefCell<Account> in future_to_promise, should only panic in multithreaded code/web workers
#![allow(clippy::await_holding_refcell_ref)]

extern crate serde;

use wasm_bindgen::prelude::*;

mod client_read_only;
pub(crate) mod wasm_types;

// Export all product_common's bindings (e.g. Transaction, CoreClient, gas-station stuff, etc).
pub use product_common::bindings::*;

/// Initializes the console error panic hook for better error messages
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

// This section should be used as the central place for imports from `./lib`
// used by TS definitions written on the Rust side.
// It appears the import path must be relative to `src`.
#[wasm_bindgen(typescript_custom_section)]
const CUSTOM_IMPORTS: &'static str = r#"
import {
  Transaction,
  TransactionOutput,
  TransactionBuilder,
  CoreClient,
  CoreClientReadOnly
} from '../lib/index';
"#;
