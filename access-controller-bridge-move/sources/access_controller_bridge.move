/// Access Controller Bridge v4 — Capability Custodian Pattern.
///
/// Bridges hierarchies' federation trust model with component authorization
/// by custodying `tf_components::Capability` objects and lending them to
/// users who pass federation validation.
///
/// The ACB is component-agnostic: it holds Capabilities for any component
/// that uses `tf_components::Capability` + `RoleMap`.
module access_controller_bridge::bridge;

use hierarchies::main::Federation;
use hierarchies::property_name::PropertyName;
use hierarchies::property_value::PropertyValue;
use tf_components::capability::{Self, Capability};
use iota::{
    clock::Clock,
    dynamic_object_field,
    event,
    vec_map::{Self, VecMap},
};
use std::string::String;

// ===== Constants =====

const PACKAGE_VERSION: u64 = 1;

// ===== Errors =====

const EVersionMismatch: u64 = 0;
const EBridgeFrozen: u64 = 1;
const ECapabilityTypeNotFound: u64 = 2;
const EFederationMismatch: u64 = 3;
const EPropertyNotProvided: u64 = 4;
const EValidationFailed: u64 = 5;
const ENotRootAuthority: u64 = 6;
const ECapabilityTypeAlreadyExists: u64 = 7;
const ECapabilityAlreadyDeposited: u64 = 8;
const ECapabilityNotDeposited: u64 = 9;
const ECapabilityTargetMismatch: u64 = 10;
const EReceiptBridgeMismatch: u64 = 11;
const EReceiptCapabilityMismatch: u64 = 12;
// reserved: 13
const EEmptyRequiredProperties: u64 = 14;
const ECapabilityCurrentlyBorrowed: u64 = 15;

// ===== Core Data Structures =====

/// The AccessControllerBridge — a capability custodian.
///
/// Stores pre-provisioned `Capability` objects as dynamic object fields
/// and lends them to users who pass federation validation via the
/// Borrow–Use–Return pattern.
///
/// Phantom P provides organizational type safety between ACB instances
/// for different component types.
public struct AccessControllerBridge<phantom P> has key, store {
    id: UID,
    /// The component instance being governed
    target_id: ID,
    /// The federation that validates trust
    federation_id: ID,
    /// Named capability types → property requirements
    capability_type_configs: VecMap<String, CapabilityTypeConfig>,
    /// Emergency freeze flag (ISO A.5.29)
    frozen: bool,
    /// Package version for migration support
    version: u64,
}

/// Configuration for a named capability type.
///
/// Defines which federation properties must be validated before lending
/// the associated Capability. Permissions are defined in the target
/// component's RoleMap, not here.
///
/// INVARIANT: required_properties MUST be non-empty.
public struct CapabilityTypeConfig has copy, drop, store {
    required_properties: vector<PropertyName>,
}

/// Dynamic object field key for stored Capabilities.
public struct CapabilityKey has copy, drop, store {
    capability_type: String,
}

/// Hot-potato receipt for a borrowed Capability.
///
/// No abilities — forces return via `return_cap()` within the same PTB.
public struct BorrowReceipt {
    capability_id: ID,
    bridge_id: ID,
    capability_type: String,
    holder: address,
}

// ===== Events =====

public struct BridgeCreated has copy, drop {
    bridge_id: ID,
    target_id: ID,
    federation_id: ID,
    created_by: address,
}

public struct CapabilityDeposited has copy, drop {
    bridge_id: ID,
    capability_type: String,
    capability_id: ID,
    capability_role: String,
    deposited_by: address,
}

public struct CapabilityWithdrawn has copy, drop {
    bridge_id: ID,
    capability_type: String,
    capability_id: ID,
    withdrawn_by: address,
}

public struct CapabilityBorrowed has copy, drop {
    bridge_id: ID,
    target_id: ID,
    capability_type: String,
    capability_id: ID,
    holder: address,
    validated_scope: VecMap<PropertyName, PropertyValue>,
    timestamp: u64,
}

public struct CapabilityReturned has copy, drop {
    bridge_id: ID,
    target_id: ID,
    capability_type: String,
    capability_id: ID,
    holder: address,
    timestamp: u64,
}

public struct BridgeUpdated has copy, drop {
    bridge_id: ID,
    updated_by: address,
    change_type: String,
}

public struct BridgeFrozen has copy, drop {
    bridge_id: ID,
    frozen_by: address,
    timestamp: u64,
}

public struct BridgeUnfrozen has copy, drop {
    bridge_id: ID,
    unfrozen_by: address,
    timestamp: u64,
}

public struct BridgeDeleted has copy, drop {
    bridge_id: ID,
    deleted_by: address,
}

// ===== CapabilityTypeConfig constructor =====

public fun new_capability_type_config(
    required_properties: vector<PropertyName>,
): CapabilityTypeConfig {
    assert!(!required_properties.is_empty(), EEmptyRequiredProperties);
    CapabilityTypeConfig { required_properties }
}

// ===== Internal Helpers =====

fun assert_root_authority<P>(
    bridge: &AccessControllerBridge<P>,
    federation: &Federation,
    ctx: &TxContext,
) {
    assert!(bridge.federation_id == object::id(federation), EFederationMismatch);
    let sender_id = ctx.sender().to_id();
    assert!(federation.is_root_authority(&sender_id), ENotRootAuthority);
}

fun assert_not_frozen<P>(bridge: &AccessControllerBridge<P>) {
    assert!(!bridge.frozen, EBridgeFrozen);
}

fun assert_version<P>(bridge: &AccessControllerBridge<P>) {
    assert!(bridge.version == PACKAGE_VERSION, EVersionMismatch);
}

// ===== Create / Delete =====

/// Create a new AccessControllerBridge. Caller must be root authority.
public fun create<P>(
    federation: &Federation,
    target_id: ID,
    capability_type_configs: VecMap<String, CapabilityTypeConfig>,
    ctx: &mut TxContext,
): AccessControllerBridge<P> {
    let sender_id = ctx.sender().to_id();
    assert!(federation.is_root_authority(&sender_id), ENotRootAuthority);

    // Validate all configs
    let keys = capability_type_configs.keys();
    let mut i = 0;
    while (i < keys.length()) {
        assert!(!capability_type_configs.get(&keys[i]).required_properties.is_empty(), EEmptyRequiredProperties);
        i = i + 1;
    };

    let federation_id = object::id(federation);
    let bridge = AccessControllerBridge<P> {
        id: object::new(ctx),
        target_id,
        federation_id,
        capability_type_configs,
        frozen: false,
        version: PACKAGE_VERSION,
    };

    event::emit(BridgeCreated {
        bridge_id: object::uid_to_inner(&bridge.id),
        target_id,
        federation_id,
        created_by: ctx.sender(),
    });

    bridge
}

/// Delete the ACB. All Capabilities must be withdrawn first.
public fun delete<P>(
    bridge: AccessControllerBridge<P>,
    federation: &Federation,
    ctx: &TxContext,
) {
    assert!(bridge.federation_id == object::id(federation), EFederationMismatch);
    let sender_id = ctx.sender().to_id();
    assert!(federation.is_root_authority(&sender_id), ENotRootAuthority);

    // Verify all Capabilities have been withdrawn
    let keys = bridge.capability_type_configs.keys();
    let mut i = 0;
    while (i < keys.length()) {
        let key = CapabilityKey { capability_type: keys[i] };
        assert!(!dynamic_object_field::exists_(&bridge.id, key), ECapabilityAlreadyDeposited);
        i = i + 1;
    };

    let bridge_id = object::uid_to_inner(&bridge.id);
    let AccessControllerBridge {
        id,
        target_id: _,
        federation_id: _,
        capability_type_configs: _,
        frozen: _,
        version: _,
    } = bridge;

    event::emit(BridgeDeleted {
        bridge_id,
        deleted_by: ctx.sender(),
    });

    object::delete(id);
}

// ===== Deposit / Withdraw =====

/// Deposit a Capability into the ACB for a specific capability type.
public fun deposit_capability<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    capability_type: String,
    cap: Capability,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    assert!(bridge.capability_type_configs.contains(&capability_type), ECapabilityTypeNotFound);
    assert!(capability::target_key(&cap) == bridge.target_id, ECapabilityTargetMismatch);

    let key = CapabilityKey { capability_type };
    assert!(!dynamic_object_field::exists_(&bridge.id, key), ECapabilityAlreadyDeposited);

    let cap_id = capability::id(&cap);
    let cap_role = *capability::role(&cap);

    dynamic_object_field::add(&mut bridge.id, key, cap);

    event::emit(CapabilityDeposited {
        bridge_id: object::uid_to_inner(&bridge.id),
        capability_type: key.capability_type,
        capability_id: cap_id,
        capability_role: cap_role,
        deposited_by: ctx.sender(),
    });
}

/// Withdraw a Capability from the ACB. Returns the Capability to the caller.
public fun withdraw_capability<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    capability_type: String,
    ctx: &TxContext,
): Capability {
    assert_root_authority(bridge, federation, ctx);

    let key = CapabilityKey { capability_type };
    assert!(dynamic_object_field::exists_(&bridge.id, key), ECapabilityNotDeposited);

    let cap: Capability = dynamic_object_field::remove(&mut bridge.id, key);

    event::emit(CapabilityWithdrawn {
        bridge_id: object::uid_to_inner(&bridge.id),
        capability_type: key.capability_type,
        capability_id: capability::id(&cap),
        withdrawn_by: ctx.sender(),
    });

    cap
}

// ===== Borrow / Return (Core Flow) =====

/// Borrow a Capability from the ACB.
///
/// Validates the caller's federation standing, removes the Capability
/// from storage, and returns it with a hot-potato BorrowReceipt.
///
/// The BorrowReceipt MUST be consumed by `return_cap()` in the same PTB.
public fun borrow<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    capability_type: String,
    property_values: VecMap<PropertyName, PropertyValue>,
    clock: &Clock,
    ctx: &TxContext,
): (Capability, BorrowReceipt) {
    assert_version(bridge);
    assert_not_frozen(bridge);
    assert!(bridge.federation_id == object::id(federation), EFederationMismatch);
    assert!(bridge.capability_type_configs.contains(&capability_type), ECapabilityTypeNotFound);

    let config = bridge.capability_type_configs.get(&capability_type);

    // Build filtered property map with only required properties
    let required = &config.required_properties;
    let mut filtered = vec_map::empty<PropertyName, PropertyValue>();
    let mut i = 0;
    while (i < required.length()) {
        let prop_name = required[i];
        assert!(property_values.contains(&prop_name), EPropertyNotProvided);
        filtered.insert(prop_name, *property_values.get(&prop_name));
        i = i + 1;
    };

    // Validate against federation
    let requester_id = ctx.sender().to_id();
    assert!(federation.validate_properties(&requester_id, filtered, clock), EValidationFailed);

    // Remove Capability from dynamic field
    let key = CapabilityKey { capability_type };
    assert!(dynamic_object_field::exists_(&bridge.id, key), ECapabilityCurrentlyBorrowed);
    let cap: Capability = dynamic_object_field::remove(&mut bridge.id, key);

    let receipt = BorrowReceipt {
        capability_id: capability::id(&cap),
        bridge_id: object::uid_to_inner(&bridge.id),
        capability_type: key.capability_type,
        holder: ctx.sender(),
    };

    event::emit(CapabilityBorrowed {
        bridge_id: object::uid_to_inner(&bridge.id),
        target_id: bridge.target_id,
        capability_type: key.capability_type,
        capability_id: capability::id(&cap),
        holder: ctx.sender(),
        validated_scope: property_values,
        timestamp: clock.timestamp_ms(),
    });

    (cap, receipt)
}

/// Return a borrowed Capability to the ACB. Consumes the hot-potato receipt.
public fun return_cap<P>(
    bridge: &mut AccessControllerBridge<P>,
    cap: Capability,
    receipt: BorrowReceipt,
    clock: &Clock,
) {
    let BorrowReceipt { capability_id, bridge_id, capability_type, holder } = receipt;

    assert!(bridge_id == object::uid_to_inner(&bridge.id), EReceiptBridgeMismatch);
    assert!(capability_id == capability::id(&cap), EReceiptCapabilityMismatch);

    let key = CapabilityKey { capability_type };
    dynamic_object_field::add(&mut bridge.id, key, cap);

    event::emit(CapabilityReturned {
        bridge_id,
        target_id: bridge.target_id,
        capability_type: key.capability_type,
        capability_id,
        holder,
        timestamp: clock.timestamp_ms(),
    });
}

// ===== Lifecycle Management =====

/// Add a new capability type configuration.
public fun add_capability_type<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    capability_type: String,
    config: CapabilityTypeConfig,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    assert!(!config.required_properties.is_empty(), EEmptyRequiredProperties);
    assert!(!bridge.capability_type_configs.contains(&capability_type), ECapabilityTypeAlreadyExists);

    bridge.capability_type_configs.insert(capability_type, config);

    event::emit(BridgeUpdated {
        bridge_id: object::uid_to_inner(&bridge.id),
        updated_by: ctx.sender(),
        change_type: b"add_capability_type".to_string(),
    });
}

/// Remove a capability type configuration. Capability must be withdrawn first.
public fun remove_capability_type<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    capability_type: String,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);

    let key = CapabilityKey { capability_type };
    assert!(!dynamic_object_field::exists_(&bridge.id, key), ECapabilityAlreadyDeposited);
    assert!(bridge.capability_type_configs.contains(&capability_type), ECapabilityTypeNotFound);

    bridge.capability_type_configs.remove(&capability_type);

    event::emit(BridgeUpdated {
        bridge_id: object::uid_to_inner(&bridge.id),
        updated_by: ctx.sender(),
        change_type: b"remove_capability_type".to_string(),
    });
}

/// Update property requirements for an existing capability type.
public fun update_capability_type_config<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    capability_type: String,
    new_config: CapabilityTypeConfig,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    assert!(!new_config.required_properties.is_empty(), EEmptyRequiredProperties);
    assert!(bridge.capability_type_configs.contains(&capability_type), ECapabilityTypeNotFound);

    bridge.capability_type_configs.remove(&capability_type);
    bridge.capability_type_configs.insert(capability_type, new_config);

    event::emit(BridgeUpdated {
        bridge_id: object::uid_to_inner(&bridge.id),
        updated_by: ctx.sender(),
        change_type: b"update_capability_type_config".to_string(),
    });
}

/// Emergency freeze — all borrow() calls fail immediately.
public fun emergency_freeze<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    clock: &Clock,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    bridge.frozen = true;

    event::emit(BridgeFrozen {
        bridge_id: object::uid_to_inner(&bridge.id),
        frozen_by: ctx.sender(),
        timestamp: clock.timestamp_ms(),
    });
}

/// Unfreeze — restores normal borrow() operation.
public fun emergency_unfreeze<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    clock: &Clock,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    bridge.frozen = false;

    event::emit(BridgeUnfrozen {
        bridge_id: object::uid_to_inner(&bridge.id),
        unfrozen_by: ctx.sender(),
        timestamp: clock.timestamp_ms(),
    });
}

/// Migrate the ACB to the current package version.
public fun migrate<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    assert!(bridge.version < PACKAGE_VERSION, EVersionMismatch);
    bridge.version = PACKAGE_VERSION;

    event::emit(BridgeUpdated {
        bridge_id: object::uid_to_inner(&bridge.id),
        updated_by: ctx.sender(),
        change_type: b"migrate".to_string(),
    });
}

// ===== Query Functions =====

public fun target_id<P>(bridge: &AccessControllerBridge<P>): ID {
    bridge.target_id
}

public fun federation_id<P>(bridge: &AccessControllerBridge<P>): ID {
    bridge.federation_id
}

public fun is_frozen<P>(bridge: &AccessControllerBridge<P>): bool {
    bridge.frozen
}

public fun has_capability_type<P>(bridge: &AccessControllerBridge<P>, capability_type: &String): bool {
    bridge.capability_type_configs.contains(capability_type)
}

public fun is_capability_deposited<P>(bridge: &AccessControllerBridge<P>, capability_type: &String): bool {
    let key = CapabilityKey { capability_type: *capability_type };
    dynamic_object_field::exists_(&bridge.id, key)
}

public fun version<P>(bridge: &AccessControllerBridge<P>): u64 {
    bridge.version
}

public fun get_required_properties(config: &CapabilityTypeConfig): &vector<PropertyName> {
    &config.required_properties
}
