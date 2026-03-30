# Access Controller Bridge

Bridges hierarchies' federation trust model with component authorization using
the **Capability Custodian Pattern**.

The ACB stores `tf_components::Capability` objects and lends them to users who
pass federation validation, enforcing mandatory return via a hot-potato
`BorrowReceipt`. The target component (e.g. audit trail) is completely unaware
of the ACB — it sees a normal `&Capability` reference.

## Architecture

```mermaid
graph TB
    subgraph Hierarchies
        Fed[Federation]
    end

    subgraph ACB["Access Controller Bridge"]
        Bridge["AccessControllerBridge&lt;P&gt;"]
        Roles["RoleConfig per role<br/>(admin-defined property values)"]
        Caps["Stored Capabilities<br/>(dynamic object fields)"]
        Bridge --- Roles
        Bridge --- Caps
    end

    subgraph Component["Target Component (unchanged)"]
        Trail["AuditTrail / any component"]
        RoleMap["RoleMap + Capability validation"]
        Trail --- RoleMap
    end

    User((User / Attester))

    User -- "1. borrow(RoleName)" --> Bridge
    Bridge -- "validate_properties()" --> Fed
    Bridge -- "2. (Capability, BorrowReceipt)" --> User
    User -- "3. &Capability" --> Trail
    Trail -- "4. operation result" --> User
    User -- "5. return_cap(Capability, BorrowReceipt)" --> Bridge
```

## Setup Flow

```mermaid
sequenceDiagram
    participant RA as Root Authority
    participant Fed as Federation
    participant Comp as Component (AuditTrail)
    participant ACB as AccessControllerBridge

    Note over RA,ACB: Phase 1-2: Create Federation & Component
    RA->>Fed: new_federation()
    Fed-->>RA: Federation (shared)
    RA->>Fed: add_property("catch_logging", [Cod, Haddock])
    RA->>Comp: create()
    Comp-->>RA: AuditTrail (shared) + admin_cap (owned)

    Note over RA,ACB: Phase 3-4: Roles & Capabilities on Component
    RA->>Comp: create_role(admin_cap, "catch_logger", {AddRecord})
    RA->>Comp: new_capability(admin_cap, "catch_logger", issued_to=None)
    Comp-->>RA: Capability (owned, role="catch_logger")

    Note over RA,ACB: Phase 5-6: Create ACB & Deposit
    RA->>ACB: create(federation, target_id, role_configs)
    Note right of ACB: role_configs:<br/>"cod_logger" → {catch_logging: "Cod"}<br/>"haddock_logger" → {catch_logging: "Haddock"}
    ACB-->>RA: ACB (shared)
    RA->>ACB: deposit_capability("cod_logger", Capability)
    Note right of ACB: Capability stored as<br/>dynamic object field

    Note over RA,ACB: Phase 7: Accredit Participants
    RA->>Fed: accredit_to_attest(fisherman, catch_logging=[Cod])
```

## Borrow-Use-Return Flow

```mermaid
sequenceDiagram
    participant User as Attester (Fisherman)
    participant ACB as AccessControllerBridge
    participant Fed as Federation
    participant Comp as Component (AuditTrail)

    Note over User,Comp: Single PTB (atomic)

    User->>ACB: borrow(PermissionContext::RoleName("cod_logger"))
    ACB->>ACB: Look up "cod_logger" → {catch_logging: "Cod"}
    ACB->>Fed: validate_properties(user, {catch_logging: "Cod"})
    Fed-->>ACB: true
    ACB->>ACB: Remove Capability from dynamic field
    ACB->>ACB: Create BorrowReceipt (no abilities = hot potato)
    ACB-->>User: (Capability, BorrowReceipt)

    User->>Comp: add_record(&Capability, data, ...)
    Comp->>Comp: assert_capability_valid(&cap, AddRecord)<br/>target_key ✓ role ✓ permission ✓
    Comp-->>User: RecordAdded

    User->>ACB: return_cap(Capability, BorrowReceipt)
    ACB->>ACB: Verify receipt.capability_id == cap.id
    ACB->>ACB: Store Capability back in dynamic field
    ACB->>ACB: Consume receipt (destructure)
    ACB-->>User: Done
    Note over User,Comp: PTB completes successfully
```

## Authorization Rejected

```mermaid
sequenceDiagram
    participant Bad as Unauthorized User
    participant ACB as AccessControllerBridge
    participant Fed as Federation

    Bad->>ACB: borrow(RoleName("cod_logger"))
    ACB->>ACB: Look up "cod_logger" → {catch_logging: "Cod"}
    ACB->>Fed: validate_properties(bad, {catch_logging: "Cod"})
    Fed-->>ACB: false (not an attester / not accredited)
    ACB->>ACB: assert! fails
    Note over Bad,Fed: ABORT — entire PTB reverts, no state changes
```

## Why BorrowReceipt?

`Capability` has `key + store` abilities (required by existing tf_components — cannot be changed).
This means a user who receives a Capability *by value* can transfer it away and keep it permanently.

The `BorrowReceipt` has **no abilities** — it cannot be dropped, stored, copied, or transferred.
The only way to consume it is `return_cap()`, which requires giving back the matching Capability.
If the user tries to keep the Capability, the unconsumed receipt aborts the PTB.

```mermaid
sequenceDiagram
    participant User as Malicious User
    participant ACB as AccessControllerBridge

    User->>ACB: borrow(RoleName("cod_logger"))
    ACB-->>User: (Capability, BorrowReceipt)
    User->>User: transfer::public_transfer(cap, my_address)
    Note over User: BorrowReceipt still alive<br/>No abilities = cannot be dropped
    Note over User,ACB: PTB ABORTS<br/>Receipt not consumed → transaction fails<br/>All state changes reverted<br/>Capability stays in ACB
```

## Lifecycle Management

```mermaid
sequenceDiagram
    participant RA as Root Authority
    participant ACB as AccessControllerBridge
    participant Fed as Federation

    Note over RA,Fed: Add a new role
    RA->>ACB: add_role("inspector", RoleConfig{catch_logging: "any"})
    RA->>ACB: deposit_capability("inspector", Capability)

    Note over RA,Fed: Update role's property values
    RA->>ACB: update_role_config("cod_logger", new_config)
    Note right of ACB: Immediate effect on next borrow()

    Note over RA,Fed: Remove a role
    RA->>ACB: withdraw_capability("inspector")
    ACB-->>RA: Capability (returned)
    RA->>ACB: remove_role("inspector")

    Note over RA,Fed: Emergency freeze
    RA->>ACB: emergency_freeze()
    Note right of ACB: All borrow() calls fail immediately
    RA->>ACB: emergency_unfreeze()
    Note right of ACB: Normal operation resumes

    Note over RA,Fed: Revoke user access (no ACB changes needed)
    RA->>Fed: revoke_accreditation(fisherman)
    Note right of Fed: Next borrow() by fisherman fails<br/>validate_properties() returns false
```

## Usage

```move
// Borrow — just name the role, property values are admin-defined
let (cap, receipt) = bridge::borrow(
    &mut acb, &fed,
    bridge::role_name(utf8(b"cod_logger")),
    &clock, ctx,
);

// Use with any component that accepts &Capability
audit_trail::add_record(&mut trail, &cap, data, metadata, tag, &clock, ctx);

// Return (mandatory — receipt is a hot potato)
bridge::return_cap(&mut acb, cap, receipt, &clock);
```

## Security Considerations

- **Operational roles must NOT include governance permissions** (AddCapabilities,
  RevokeCapabilities, AddRoles, etc.). A user could otherwise mint themselves a
  persistent Capability during the borrow window.
- Governance operations should use the **direct path** — root authorities hold
  their own admin Capability and call the component directly.
- Admin defines exact property name+value pairs per role. The borrower cannot
  influence what values are validated — eliminating caller-chosen scope as an
  attack surface.

## Building

```bash
iota move build
```

## Testing

```bash
iota move test
```

## Dependencies

- `Hierarchies` — federation validation (`validate_properties`, `is_root_authority`)
- `TfComponents` — `Capability` struct and accessors
