# Access Controller Bridge — TypeScript Examples

Runnable examples demonstrating the Access Controller Bridge lifecycle using the `@iota/iota-sdk` TypeScript SDK against a live IOTA network.

## Prerequisites

1. Node.js >= 20

2. A running IOTA localnet (or devnet/testnet):
   ```bash
   iota start --force-regenesis
   ```

3. Published packages (see [Rust examples README](../../rs/examples/README.md#prerequisites) for the full publish sequence). Record all four package IDs.

4. Install dependencies:
   ```bash
   npm install
   ```

## Running

Export all four package IDs and run an example:

```bash
export IOTA_HIERARCHIES_PKG_ID=0x...
export IOTA_AUDIT_TRAIL_PKG_ID=0x...
export IOTA_TF_COMPONENTS_PKG_ID=0x...
export IOTA_ACB_PKG_ID=0x...

npx ts-node 01_full_initialization.ts
npx ts-node 02_borrow_use_return.ts
```

Or via npm scripts:

```bash
npm run example:init
npm run example:borrow
```

### Optional Environment Variables

| Variable | Default | Description |
|---|---|---|
| `NETWORK_URL` | `http://127.0.0.1:9000` | IOTA node RPC endpoint |
| `NETWORK_NAME_FAUCET` | `localnet` | Faucet network name |

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
        → ACB validates federation standing using admin-defined property values
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
ts/examples/
├── package.json                  # @iota/iota-sdk dependency
├── tsconfig.json                 # ES2022, CommonJS
├── utils.ts                      # PtbHelper — PTB construction for all Move calls
├── 01_full_initialization.ts     # Full 7-phase setup
├── 02_borrow_use_return.ts       # Borrow-Use-Return flow
└── README.md
```

## Key Pattern: PtbHelper

All raw PTB construction is encapsulated in the `PtbHelper` class (utils.ts). Key methods:

| Method | Description |
|---|---|
| `acbCreate()` | Create ACB with role configs (name + property name-value pairs) |
| `acbShare()` | Share the ACB object |
| `acbDeposit()` | Deposit a Capability for a role |
| `acbBorrow()` | Construct `PermissionContext::RoleName` and call `borrow()` |
| `acbReturn()` | Return Capability + consume receipt |
| `propertyMap()` | Build `VecMap<PropertyName, PropertyValue>` |
| `trailCreate()` | Create audit trail |
| `trailCreateRole()` | Create role with permissions |
| `trailMintCapability()` | Mint bearer Capability |
| `trailAddRecord()` | Add record to audit trail |

## Note: Transaction Indexing

Each `executeTx()` call includes `client.waitForTransaction()` after execution. This ensures the indexer has processed the transaction before the next one tries to resolve object references. Without this, `tx.object(id)` can fail because the object version isn't indexed yet.
