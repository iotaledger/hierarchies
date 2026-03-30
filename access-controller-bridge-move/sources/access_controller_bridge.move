/// Access Controller Bridge v4.1 — Capability Custodian Pattern
/// with Role-Based Property Mapping.
///
/// Bridges hierarchies' federation trust model with component authorization
/// by custodying `tf_components::Capability` objects and lending them to
/// users who pass federation validation.
///
/// Admin defines exact property name+value pairs per role. Borrowers provide
/// only a `PermissionContext::RoleName` — they cannot influence what values
/// are validated.
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
    vec_map::VecMap,
};
use std::string::String;

// ===== Constants =====

const PACKAGE_VERSION: u64 = 1;

// ===== Errors =====

const EVersionMismatch: u64 = 0;
const EBridgeFrozen: u64 = 1;
const ERoleNotFound: u64 = 2;
const EFederationMismatch: u64 = 3;
// reserved: 4 (was EPropertyNotProvided — no longer needed)
const EValidationFailed: u64 = 5;
const ENotRootAuthority: u64 = 6;
const ERoleAlreadyExists: u64 = 7;
const ECapabilityAlreadyDeposited: u64 = 8;
const ECapabilityNotDeposited: u64 = 9;
const ECapabilityTargetMismatch: u64 = 10;
const EReceiptBridgeMismatch: u64 = 11;
const EReceiptCapabilityMismatch: u64 = 12;
// reserved: 13
const EEmptyPropertyValues: u64 = 14;
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
///
/// IMPORTANT: The ACB has `store` ability because `create()` returns it
/// by value and the caller must `transfer::public_share_object()` it,
/// which requires `store`. The ACB MUST be shared immediately after
/// creation. If kept as an owned object, the owner controls all access
/// and the federation-mediated model is bypassed.
public struct AccessControllerBridge<phantom P> has key, store {
    id: UID,
    /// The component instance being governed
    target_id: ID,
    /// The federation that validates trust
    federation_id: ID,
    /// Named roles → property value requirements
    role_configs: VecMap<String, RoleConfig>,
    /// Emergency freeze flag (ISO A.5.29)
    frozen: bool,
    /// Package version for migration support
    version: u64,
}

/// Configuration for a named role.
///
/// Defines the exact property name+value pairs that the federation must
/// validate before lending the associated Capability. The admin defines
/// both the property names AND values — the borrower does not choose values.
///
/// Permissions are defined in the target component's RoleMap, not here.
///
/// INVARIANT: property_values MUST be non-empty.
public struct RoleConfig has copy, drop, store {
    property_values: VecMap<PropertyName, PropertyValue>,
}

/// Enum representing the context for a permission request.
///
/// The borrower passes a PermissionContext to borrow() instead of
/// a raw string + property values. Self-documenting and extensible.
public enum PermissionContext has copy, drop, store {
    RoleName(String),
}

/// Dynamic object field key for stored Capabilities.
public struct CapabilityKey has copy, drop, store {
    role_name: String,
}

/// Hot-potato receipt for a borrowed Capability.
///
/// No abilities — forces return via `return_cap()` within the same PTB.
public struct BorrowReceipt {
    capability_id: ID,
    bridge_id: ID,
    role_name: String,
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
    role_name: String,
    capability_id: ID,
    capability_role: String,
    deposited_by: address,
}

public struct CapabilityWithdrawn has copy, drop {
    bridge_id: ID,
    role_name: String,
    capability_id: ID,
    withdrawn_by: address,
}

public struct CapabilityBorrowed has copy, drop {
    bridge_id: ID,
    target_id: ID,
    role_name: String,
    capability_id: ID,
    holder: address,
    validated_scope: VecMap<PropertyName, PropertyValue>,
    timestamp: u64,
}

public struct CapabilityReturned has copy, drop {
    bridge_id: ID,
    target_id: ID,
    role_name: String,
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

// ===== Constructors =====

public fun new_role_config(
    property_values: VecMap<PropertyName, PropertyValue>,
): RoleConfig {
    assert!(!property_values.is_empty(), EEmptyPropertyValues);
    RoleConfig { property_values }
}

public fun role_name(name: String): PermissionContext {
    PermissionContext::RoleName(name)
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
    role_configs: VecMap<String, RoleConfig>,
    ctx: &mut TxContext,
): AccessControllerBridge<P> {
    let sender_id = ctx.sender().to_id();
    assert!(federation.is_root_authority(&sender_id), ENotRootAuthority);

    // Validate all configs
    let keys = role_configs.keys();
    let mut i = 0;
    while (i < keys.length()) {
        assert!(!role_configs.get(&keys[i]).property_values.is_empty(), EEmptyPropertyValues);
        i = i + 1;
    };

    let federation_id = object::id(federation);
    let bridge = AccessControllerBridge<P> {
        id: object::new(ctx),
        target_id,
        federation_id,
        role_configs,
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
    let keys = bridge.role_configs.keys();
    let mut i = 0;
    while (i < keys.length()) {
        let key = CapabilityKey { role_name: keys[i] };
        assert!(!dynamic_object_field::exists_(&bridge.id, key), ECapabilityAlreadyDeposited);
        i = i + 1;
    };

    let bridge_id = object::uid_to_inner(&bridge.id);
    let AccessControllerBridge {
        id,
        target_id: _,
        federation_id: _,
        role_configs: _,
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

/// Deposit a Capability into the ACB for a specific role.
public fun deposit_capability<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    role_name: String,
    cap: Capability,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    assert!(bridge.role_configs.contains(&role_name), ERoleNotFound);
    assert!(capability::target_key(&cap) == bridge.target_id, ECapabilityTargetMismatch);

    let key = CapabilityKey { role_name };
    assert!(!dynamic_object_field::exists_(&bridge.id, key), ECapabilityAlreadyDeposited);

    let cap_id = capability::id(&cap);
    let cap_role = *capability::role(&cap);

    dynamic_object_field::add(&mut bridge.id, key, cap);

    event::emit(CapabilityDeposited {
        bridge_id: object::uid_to_inner(&bridge.id),
        role_name: key.role_name,
        capability_id: cap_id,
        capability_role: cap_role,
        deposited_by: ctx.sender(),
    });
}

/// Withdraw a Capability from the ACB. Returns the Capability to the caller.
public fun withdraw_capability<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    role_name: String,
    ctx: &TxContext,
): Capability {
    assert_root_authority(bridge, federation, ctx);

    let key = CapabilityKey { role_name };
    assert!(dynamic_object_field::exists_(&bridge.id, key), ECapabilityNotDeposited);

    let cap: Capability = dynamic_object_field::remove(&mut bridge.id, key);

    event::emit(CapabilityWithdrawn {
        bridge_id: object::uid_to_inner(&bridge.id),
        role_name: key.role_name,
        capability_id: capability::id(&cap),
        withdrawn_by: ctx.sender(),
    });

    cap
}

// ===== Borrow / Return (Core Flow) =====

/// Borrow a Capability from the ACB.
///
/// The borrower provides a `PermissionContext` (currently: a role name).
/// The ACB looks up the role's pre-defined property values, validates
/// them against the federation, and lends the Capability.
///
/// The BorrowReceipt MUST be consumed by `return_cap()` in the same PTB.
public fun borrow<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    context: PermissionContext,
    clock: &Clock,
    ctx: &TxContext,
): (Capability, BorrowReceipt) {
    assert_version(bridge);
    assert_not_frozen(bridge);
    assert!(bridge.federation_id == object::id(federation), EFederationMismatch);

    // Extract role name from PermissionContext
    let role_name = match (&context) {
        PermissionContext::RoleName(name) => *name,
    };

    assert!(bridge.role_configs.contains(&role_name), ERoleNotFound);
    let config = bridge.role_configs.get(&role_name);

    // Validate admin-defined property values against the federation
    let requester_id = ctx.sender().to_id();
    let property_values = config.property_values;
    assert!(
        federation.validate_properties(&requester_id, property_values, clock),
        EValidationFailed,
    );

    // Remove Capability from dynamic field
    let key = CapabilityKey { role_name };
    assert!(dynamic_object_field::exists_(&bridge.id, key), ECapabilityCurrentlyBorrowed);
    let cap: Capability = dynamic_object_field::remove(&mut bridge.id, key);

    let receipt = BorrowReceipt {
        capability_id: capability::id(&cap),
        bridge_id: object::uid_to_inner(&bridge.id),
        role_name: key.role_name,
        holder: ctx.sender(),
    };

    event::emit(CapabilityBorrowed {
        bridge_id: object::uid_to_inner(&bridge.id),
        target_id: bridge.target_id,
        role_name: key.role_name,
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
    let BorrowReceipt { capability_id, bridge_id, role_name, holder } = receipt;

    assert!(bridge_id == object::uid_to_inner(&bridge.id), EReceiptBridgeMismatch);
    assert!(capability_id == capability::id(&cap), EReceiptCapabilityMismatch);

    let key = CapabilityKey { role_name };
    dynamic_object_field::add(&mut bridge.id, key, cap);

    event::emit(CapabilityReturned {
        bridge_id,
        target_id: bridge.target_id,
        role_name: key.role_name,
        capability_id,
        holder,
        timestamp: clock.timestamp_ms(),
    });
}

// ===== Lifecycle Management =====

/// Add a new role configuration.
public fun add_role<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    role_name: String,
    config: RoleConfig,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    assert!(!config.property_values.is_empty(), EEmptyPropertyValues);
    assert!(!bridge.role_configs.contains(&role_name), ERoleAlreadyExists);

    bridge.role_configs.insert(role_name, config);

    event::emit(BridgeUpdated {
        bridge_id: object::uid_to_inner(&bridge.id),
        updated_by: ctx.sender(),
        change_type: b"add_role".to_string(),
    });
}

/// Remove a role configuration. Capability must be withdrawn first.
public fun remove_role<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    role_name: String,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);

    let key = CapabilityKey { role_name };
    assert!(!dynamic_object_field::exists_(&bridge.id, key), ECapabilityAlreadyDeposited);
    assert!(bridge.role_configs.contains(&role_name), ERoleNotFound);

    bridge.role_configs.remove(&role_name);

    event::emit(BridgeUpdated {
        bridge_id: object::uid_to_inner(&bridge.id),
        updated_by: ctx.sender(),
        change_type: b"remove_role".to_string(),
    });
}

/// Add a new role and deposit its Capability in one call.
public fun add_role_with_capability<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    role_name: String,
    config: RoleConfig,
    cap: Capability,
    ctx: &TxContext,
) {
    add_role(bridge, federation, role_name, config, ctx);
    deposit_capability(bridge, federation, role_name, cap, ctx);
}

/// Withdraw a Capability and remove its role config in one call.
public fun remove_role_and_withdraw<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    role_name: String,
    ctx: &TxContext,
): Capability {
    let cap = withdraw_capability(bridge, federation, role_name, ctx);
    remove_role(bridge, federation, role_name, ctx);
    cap
}

/// Update property values for an existing role.
public fun update_role_config<P>(
    bridge: &mut AccessControllerBridge<P>,
    federation: &Federation,
    role_name: String,
    new_config: RoleConfig,
    ctx: &TxContext,
) {
    assert_root_authority(bridge, federation, ctx);
    assert!(!new_config.property_values.is_empty(), EEmptyPropertyValues);
    assert!(bridge.role_configs.contains(&role_name), ERoleNotFound);

    bridge.role_configs.remove(&role_name);
    bridge.role_configs.insert(role_name, new_config);

    event::emit(BridgeUpdated {
        bridge_id: object::uid_to_inner(&bridge.id),
        updated_by: ctx.sender(),
        change_type: b"update_role_config".to_string(),
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

public fun has_role<P>(bridge: &AccessControllerBridge<P>, role_name: &String): bool {
    bridge.role_configs.contains(role_name)
}

public fun is_capability_deposited<P>(bridge: &AccessControllerBridge<P>, role_name: &String): bool {
    let key = CapabilityKey { role_name: *role_name };
    dynamic_object_field::exists_(&bridge.id, key)
}

public fun version<P>(bridge: &AccessControllerBridge<P>): u64 {
    bridge.version
}

public fun get_property_values(config: &RoleConfig): &VecMap<PropertyName, PropertyValue> {
    &config.property_values
}
