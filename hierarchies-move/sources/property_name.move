module hierarchies::property_name;

use std::string::String;

/// PropertyName represents a name of a Property. It can be a single name or a vector of names.
public struct PropertyName has copy, drop, store {
    names: vector<String>,
}

public fun new_property_name(v: String): PropertyName {
    let mut names = vector::empty();
    names.push_back(v);
    PropertyName {
        names,
    }
}

public fun new_property_name_from_vector(v: vector<String>): PropertyName {
    PropertyName {
        names: v,
    }
}

public fun names(self: &PropertyName): &vector<String> {
    &self.names
}
