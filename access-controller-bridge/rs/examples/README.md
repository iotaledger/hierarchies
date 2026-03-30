# Access Controller Bridge — Rust Examples

Runnable examples demonstrating the Access Controller Bridge lifecycle against a live IOTA network.

## Prerequisites

1. A running IOTA localnet (or devnet/testnet):
   ```bash
   iota start --force-regenesis
   ```

2. Published packages. The ACB depends on Hierarchies and TfComponents (which are resolved
   via `Move.toml` local paths). Publish the ACB with `--with-unpublished-dependencies` to
   publish all dependencies in one step, or publish them individually in dependency order:
   ```bash
   # From the repo root — publish ACB (includes dependencies automatically)
   cd access-controller-bridge/move
   iota client publish --with-unpublished-dependencies --silence-warnings --json --gas-budget 500000000
   # Record IOTA_ACB_PKG_ID from the output

   # The audit trail is a separate package (not a dependency of the ACB).
   # It must be published separately if you want to run examples that use it.
   # See the audit trail repo for publish instructions.
   # Record IOTA_AUDIT_TRAIL_PKG_ID

   # Extract the remaining package IDs from the ACB publish output:
   # IOTA_HIERARCHIES_PKG_ID and IOTA_TF_COMPONENTS_PKG_ID appear in the
   # objectChanges as "published" entries for the dependency packages.
   ```

## Running

Export all four package IDs and run an example:

```bash
export IOTA_HIERARCHIES_PKG_ID=0x...
export IOTA_AUDIT_TRAIL_PKG_ID=0x...
export IOTA_TF_COMPONENTS_PKG_ID=0x...
export IOTA_ACB_PKG_ID=0x...

cargo run --example 01_full_initialization
cargo run --example 02_borrow_use_return
```

## Examples

### 01_full_initialization

Complete 7-phase setup of the Access Controller Bridge:

1. Create federation and add `catch_logging` property
2. Create audit trail
3. Create `catch_logger` role on the audit trail
4. Mint a bearer Capability for `catch_logger`
5. Create ACB with role config: `catch_logger` requires `{catch_logging: "Cod"}`
6. Deposit the Capability into the ACB
7. Accredit the caller as an attester for `catch_logging = [Cod, Haddock]`

Prints all created object IDs.

### 02_borrow_use_return

Self-contained example that runs the full setup, then demonstrates the core **Borrow-Use-Return** flow in a single PTB:

```
Step 1: borrow(RoleName("catch_logger"))
        → ACB validates federation standing
        → Returns (Capability, BorrowReceipt)

Step 2: add_record(&Capability, "Cod catch logged via ACB")
        → Audit trail validates Capability as usual
        → Record added

Step 3: return_cap(Capability, BorrowReceipt)
        → Capability stored back in ACB
        → Receipt consumed (hot potato)
```

All three steps execute atomically in one PTB. If any step fails, the entire transaction aborts.

## Project Structure

```
access-controller-bridge/rs/examples/
├── Cargo.toml                  # Workspace member, depends on iota-sdk + hierarchies
├── utils/
│   └── utils.rs                # PtbHelper — PTB construction for all Move calls
├── 01_full_initialization.rs   # Full 7-phase setup
├── 02_borrow_use_return.rs     # Borrow-Use-Return flow
└── README.md
```

Run from this directory (`access-controller-bridge/rs/examples/`).

## Key Pattern: PtbHelper

All raw PTB construction is encapsulated in `PtbHelper` (utils.rs). Key methods:

| Method | Description |
|---|---|
| `acb_create()` | Create ACB with role configs (name + property name-value pairs) |
| `acb_deposit()` | Deposit a Capability for a role |
| `acb_borrow()` | Construct `PermissionContext::RoleName` and call `borrow()` |
| `acb_return()` | Return Capability + consume receipt |
| `property_map()` | Build `VecMap<PropertyName, PropertyValue>` |
| `trail_create()` | Create audit trail |
| `trail_create_role()` | Create role with permissions |
| `trail_mint_capability()` | Mint bearer Capability |
| `trail_add_record()` | Add record to audit trail |
