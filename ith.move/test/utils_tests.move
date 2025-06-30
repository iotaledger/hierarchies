// Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// This module provides tests for the timelock module
#[test_only]
module ith::util_tests;

use iota::{vec_map, vec_set};
use ith::utils;

#[test]
fun test_vec_map_from_keys_values_works() {
    let addresses = vector[@0x1, @0x2];
    let vps = vector[1, 1];

    let map = utils::vec_map_from_keys_values(addresses, vps);
    assert!(vec_map::size(&map) == 2, 0);
    assert!(vec_map::contains(&map, &@0x1), 1);
    assert!(vec_map::contains(&map, &@0x2), 2);
    assert!(*vec_map::get(&map, &@0x1) == 1, 3);
    assert!(*vec_map::get(&map, &@0x2) == 1, 4);
}

#[test]
fun test_vec_map_from_keys_values_empty() {
    let addresses: vector<address> = vector[];
    let vps: vector<u64> = vector[];

    let map = utils::vec_map_from_keys_values(addresses, vps);
    assert!(vec_map::size(&map) == 0, 0);
}

#[test]
#[expected_failure(abort_code = 0)] // ELengthMismatch
fun test_vec_map_from_keys_values_length_mismatch() {
    let addresses = vector[@0x1, @0x2];
    let vps = vector[1]; // Different length

    let _map = utils::vec_map_from_keys_values(addresses, vps);
}

#[test]
fun test_contains_one_of_true() {
    let source = vector[1, 2, 3, 4];
    let one_of = vector[3, 5, 6];

    assert!(utils::contains_one_of(&source, &one_of), 0);
}

#[test]
fun test_contains_one_of_false() {
    let source = vector[1, 2, 3, 4];
    let one_of = vector[5, 6, 7];

    assert!(!utils::contains_one_of(&source, &one_of), 0);
}

#[test]
fun test_contains_one_of_empty_one_of() {
    let source = vector[1, 2, 3];
    let one_of: vector<u64> = vector[];

    // Note: Based on the implementation, this should return true since the while loop doesn't execute
    assert!(utils::contains_one_of(&source, &one_of), 0);
}

#[test]
fun test_contains_one_of_empty_source() {
    let source: vector<u64> = vector[];
    let one_of = vector[1, 2, 3];

    assert!(utils::contains_one_of(&source, &one_of), 0);
}

#[test]
fun test_contains_all_from_true() {
    let source = vector[1, 2, 3, 4, 5];
    let all_from = vector[1, 3, 5];

    assert!(utils::contains_all_from(&source, &all_from), 0);
}

#[test]
fun test_contains_all_from_false() {
    let source = vector[1, 2, 3];
    let all_from = vector[1, 2, 4]; // 4 is not in source

    assert!(!utils::contains_all_from(&source, &all_from), 0);
}

#[test]
fun test_contains_all_from_empty_all_from() {
    let source = vector[1, 2, 3];
    let all_from: vector<u64> = vector[];

    assert!(utils::contains_all_from(&source, &all_from), 0);
}

#[test]
fun test_contains_all_from_empty_source() {
    let source: vector<u64> = vector[];
    let all_from = vector[1];

    assert!(!utils::contains_all_from(&source, &all_from), 0);
}

#[test]
fun test_copy_vector() {
    let original = vector[1, 2, 3, 4];
    let copied = utils::copy_vector(&original);

    assert!(vector::length(&copied) == 4, 0);
    assert!(copied[0] == 1, 1);
    assert!(copied[1] == 2, 2);
    assert!(copied[2] == 3, 3);
    assert!(copied[3] == 4, 4);

    // Verify they are separate vectors by modifying original
    vector::push_back(&mut original, 5);
    assert!(vector::length(&original) == 5, 5);
    assert!(vector::length(&copied) == 4, 6); // copied should remain unchanged
}

#[test]
fun test_copy_vector_empty() {
    let original: vector<u64> = vector[];
    let copied = utils::copy_vector(&original);

    assert!(vector::length(&copied) == 0, 0);
}

#[test]
fun test_create_vec_set() {
    let values = vector[1, 2, 3, 2, 1]; // Contains duplicates
    let set = utils::create_vec_set(values);

    assert!(vec_set::size(&set) == 3, 0); // Should contain only unique values
    assert!(vec_set::contains(&set, &1), 1);
    assert!(vec_set::contains(&set, &2), 2);
    assert!(vec_set::contains(&set, &3), 3);
}

#[test]
fun test_create_vec_set_empty() {
    let values: vector<u64> = vector[];
    let set = utils::create_vec_set(values);

    assert!(vec_set::size(&set) == 0, 0);
}

#[test]
fun test_create_vec_set_no_duplicates() {
    let values = vector[1, 2, 3];
    let set = utils::create_vec_set(values);

    assert!(vec_set::size(&set) == 3, 0);
    assert!(vec_set::contains(&set, &1), 1);
    assert!(vec_set::contains(&set, &2), 2);
    assert!(vec_set::contains(&set, &3), 3);
}
