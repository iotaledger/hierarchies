// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use product_common::move_history_manager::MoveHistoryManager;

fn main() {
    let move_lock_path = "../../hierarchies-move/Move.lock";
    println!("[build.rs] move_lock_path: {move_lock_path}");
    let move_history_path = "../../hierarchies-move/Move.history.json";
    println!("[build.rs] move_history_path: {move_history_path}");

    MoveHistoryManager::new(
        &PathBuf::from(move_lock_path),
        &PathBuf::from(move_history_path),
        // We will watch the default watch list (`get_default_aliases_to_watch()`) in this build script
        // so we leave the `additional_aliases_to_watch` argument vec empty.
        // Use for example `vec!["localnet".to_string()]` instead, if you don't want to ignore `localnet`.
        vec![],
    )
    .manage_history_file(|message| {
        println!("[build.rs] {}", message);
    })
    .expect("Successfully managed Move history file");

    // Tell Cargo to rerun this build script if the Move.lock file changes.
    println!("cargo::rerun-if-changed={move_lock_path}");
}