module hierarchies::accreditation;

use hierarchies::{
    property::{Self, FederationProperty},
    property_name::PropertyName,
    property_value::PropertyValue,
    utils
};
use iota::vec_map::VecMap;
use std::string::String;

public struct Accreditations has store {
    accreditations: vector<Accreditation>,
}

/// Creates a new empty list of Accreditations.
public fun new_empty_accreditations(): Accreditations {
    Accreditations {
        accreditations: vector::empty(),
    }
}

/// Creates a collection of Accreditations.
public fun new_accreditations(statements: vector<Accreditation>): Accreditations {
    Accreditations {
        accreditations: statements,
    }
}

/// Adds an accredited property to the list of accreditations.
public(package) fun add_accreditation(
    self: &mut Accreditations,
    accredited_statement: Accreditation,
) {
    self.accreditations.push_back(accredited_statement);
}

/// Check if the properties are allowed by any of the accredited properties.
/// The properties are allowed if the accredited property is not expired and the property value matches the condition.
public(package) fun are_properties_allowed(
    self: &Accreditations,
    properties: &VecMap<PropertyName, PropertyValue>,
    current_time_ms: u64,
): bool {
    let property_names = properties.keys();
    let mut idx_property_names = 0;

    while (idx_property_names < property_names.length()) {
        let property_name = property_names[idx_property_names];
        let property_value = properties.get(&property_name);

        if (!self.is_property_allowed(&property_name, property_value, current_time_ms)) {
            return false
        };
        idx_property_names = idx_property_names + 1;
    };
    return true
}

/// Check if the property is allowed by any of the accredited properties.
public(package) fun is_property_allowed(
    self: &Accreditations,
    property_name: &PropertyName,
    property_value: &PropertyValue,
    current_time_ms: u64,
): bool {
    let len_properties_to_attest = self.accreditations.length();
    let mut idx_properties_to_attest = 0;

    while (idx_properties_to_attest < len_properties_to_attest) {
        let accreditation = &self.accreditations[idx_properties_to_attest];
        let maybe_property = accreditation.properties.try_get(property_name);

        if (maybe_property.is_none()) {
            continue
        };
        if (
            maybe_property
                .borrow()
                .matches_name_value(property_name, property_value, current_time_ms)
        ) {
            return true
        };
        idx_properties_to_attest = idx_properties_to_attest + 1;
    };
    return false
}

/// Check the compliance of the properties. The compliance is met if all set of properties names and values is at most the set of accredited properties.
public(package) fun are_properties_compliant(
    self: &Accreditations,
    properties: &vector<FederationProperty>,
    current_time_ms: u64,
): bool {
    let mut idx = 0;
    while (idx < properties.length()) {
        let property = properties[idx];
        if (!self.is_property_compliant(&property, current_time_ms)) {
            return false
        };
        idx = idx + 1;
    };
    return true
}

/// Check the compliance of the property. The compliance is met if all set of property names and values is at most the set of accredited properties.
public(package) fun is_property_compliant(
    self: &Accreditations,
    property: &FederationProperty,
    current_time_ms: u64,
): bool {
    let len_statements = self.accreditations.length();
    let mut idx_statements = 0;
    let mut want_properties: vector<PropertyValue> = utils::copy_vector(property
        .allowed_values()
        .keys());

    while (idx_statements < len_statements) {
        let accredited_statement = &self.accreditations[idx_statements];

        let value_condition = accredited_statement.properties.try_get(property.property_name());
        if (value_condition.is_none()) {
            idx_statements = idx_statements + 1;
            continue
        };

        // TODO
        // This is make sure the names are checked when we remove VecMap to store the Statements in the Accreditation.
        // Check if the accredited statement matches the statement name
        if (!value_condition.borrow().matches_name(property.property_name())) {
            idx_statements = idx_statements + 1;
            continue
        };

        // Check each required value against the accredited statement
        let mut len_want_properties = want_properties.length();
        let mut idx_want_properties = 0;
        while (idx_want_properties < len_want_properties) {
            let property_value = want_properties[idx_want_properties];
            if (value_condition.borrow().matches_value(&property_value, current_time_ms)) {
                // Remove the matched value from the want list
                want_properties.remove(idx_want_properties);
                len_want_properties = len_want_properties - 1;
                // Don't increment idx_want_properties because the next element now has the same index
            } else {
                idx_want_properties = idx_want_properties + 1;
            };
        };
        idx_statements = idx_statements + 1;
    };

    // All wanted properties have been found
    if (want_properties.length() == 0) {
        return true
    };
    return false
}

public(package) fun accredited_properties(self: &Accreditations): &vector<Accreditation> {
    &self.accreditations
}

public(package) fun remove_accredited_property(self: &mut Accreditations, accreditation_id: &ID) {
    let mut idx = self.find_accredited_property_id(accreditation_id);
    if (idx.is_none()) {
        return
    };
    let Accreditation {
        id: uid,
        properties: _,
        accredited_by: _,
    } = self.accreditations.remove(idx.extract());
    object::delete(uid);
}

public(package) fun find_accredited_property_id(self: &Accreditations, id: &ID): Option<u64> {
    let mut idx = 0;
    while (idx < self.accreditations.length()) {
        if (self.accreditations[idx].id.to_inner() == *id) {
            return option::some(idx)
        };
        idx = idx + 1;
    };
    option::none()
}

/// Accreditation represents statements that are accredited by a third party.
public struct Accreditation has key, store {
    id: UID,
    accredited_by: String,
    properties: VecMap<PropertyName, FederationProperty>,
}

public fun new_accreditation(properties: vector<FederationProperty>, ctx: &mut TxContext): Accreditation {
    let properties_map = property::to_map_of_properties(properties);

    Accreditation {
        id: object::new(ctx),
        accredited_by: ctx.sender().to_string(),
        properties: properties_map,
    }
}

public(package) fun id(self: &Accreditation): &UID {
    &self.id
}

public(package) fun accredited_by(self: &Accreditation): &String {
    &self.accredited_by
}

public(package) fun properties(self: &Accreditation): &VecMap<PropertyName, FederationProperty> {
    &self.properties
}

// ===== Test-only Functions =====

#[test_only]
public(package) fun destroy_accreditation(self: Accreditation) {
    let Accreditation {
        id: id,
        accredited_by: _,
        properties: _,
    } = self;

    object::delete(id);
}

#[test_only]
public(package) fun destroy_accreditations(self: Accreditations) {
    let Accreditations { accreditations } = self;

    accreditations.destroy!(|accreditation| {
        destroy_accreditation(accreditation);
    });
}
