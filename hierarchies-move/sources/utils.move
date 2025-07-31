module hierarchies::utils;

use iota::{vec_map::{Self, VecMap}, vec_set::{Self, VecSet}};

// ======= Error Codes =======
const ELengthMismatch: u64 = 0;

public(package) fun contains_one_of<D: copy + drop>(source: &vector<D>, one_of: &vector<D>): bool {
    let len_one_of = vector::length<D>(one_of);
    // If one_of is empty, there's nothing to match, so return false
    if (len_one_of == 0) {
        return false
    };
    let mut idx_one_of = 0;
    while (idx_one_of < len_one_of) {
        if (vector::contains<D>(source, &one_of[idx_one_of])) {
            return true
        };
        idx_one_of = idx_one_of + 1;
    };
    return false
}

public(package) fun contains_all_from<D: copy + drop>(
    source: &vector<D>,
    all_from: &vector<D>,
): bool {
    // if encounter ANY mistake, return false
    let len_all_from = all_from.length();
    let mut idx_all_from = 0;
    while (idx_all_from < len_all_from) {
        if (!source.contains(&all_from[idx_all_from])) {
            return false
        };
        idx_all_from = idx_all_from + 1;
    };
    return true
}

public fun copy_vector<D: copy>(src: &vector<D>): vector<D> {
    let mut idx = 0;
    let mut cloned: vector<D> = vector::empty();
    while (idx < src.length()) {
        cloned.push_back(src[idx]);
        idx = idx + 1;
    };
    cloned
}

public fun create_vec_set<T: copy + drop + store>(mut values: vector<T>): VecSet<T> {
    let mut set = vec_set::empty();
    while (!vector::is_empty(&values)) {
        let value = vector::pop_back(&mut values);
        vec_set::insert(&mut set, value);
    };

    values.destroy_empty();
    set
}

public fun vec_map_from_keys_values<K: store + copy, V: store>(
    mut keys: vector<K>,
    mut values: vector<V>,
): VecMap<K, V> {
    assert!(keys.length() == values.length(), ELengthMismatch);

    let mut map = vec_map::empty<K, V>();
    while (!keys.is_empty()) {
        let key = keys.swap_remove(0);
        let value = values.swap_remove(0);
        map.insert(key, value);
    };
    keys.destroy_empty();
    values.destroy_empty();

    map
}
