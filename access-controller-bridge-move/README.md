# Access Controller Bridge

Bridges hierarchies' federation trust model with component authorization using
the **Capability Custodian Pattern**.

The ACB stores `tf_components::Capability` objects and lends them to users who
pass federation validation, enforcing mandatory return via a hot-potato
`BorrowReceipt`. The target component (e.g. audit trail) is completely unaware
of the ACB — it sees a normal `&Capability` reference.

## Architecture

```
User ── borrow() ──► ACB ── validate_properties() ──► Federation
     ◄─ (Capability,    │
        BorrowReceipt) ─┘
     │
     ├── &Capability ──► Component (unchanged)
     │
     └── return_cap() ──► ACB
```

All three steps happen in a single PTB (Programmable Transaction Block).

## Setup

1. Create a federation and add properties (hierarchies)
2. Create the target component (e.g. audit trail)
3. Create roles on the component's RoleMap
4. Mint bearer Capabilities for those roles (`issued_to: None`)
5. Create the AccessControllerBridge with capability type configs
6. Deposit the Capabilities into the ACB
7. Accredit participants in the federation

## Usage (Borrow–Use–Return)

```move
// 1. Borrow
let (cap, receipt) = bridge::borrow(&mut acb, &fed, type, props, &clock, ctx);

// 2. Use with any component that accepts &Capability
component::operation(&mut obj, &cap, ...);

// 3. Return (mandatory — receipt is a hot potato)
bridge::return_cap(&mut acb, cap, receipt, &clock);
```

## Security Considerations

- **Operational roles must NOT include governance permissions** (AddCapabilities,
  RevokeCapabilities, AddRoles, etc.). A user could otherwise mint themselves a
  persistent Capability during the borrow window.
- Governance operations should use the **direct path** — root authorities hold
  their own admin Capability and call the component directly.
- The `BorrowReceipt` has no abilities. It cannot be dropped, stored, copied, or
  transferred. The only way to consume it is `return_cap()`.

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
