# IOTA Hierarchies WASM Examples

The following code examples demonstrate how to use IOTA Hierarchies WASM bindings for creating, managing, and interacting with hierarchical trust networks in TypeScript/JavaScript applications.

## Prerequisites

Examples can be run against:

- A local IOTA node
- An existing network, e.g., the IOTA testnet

When setting up a local node, you'll need to publish a hierarchies package as described in the IOTA documentation. You'll also need to provide environment variables for your locally deployed hierarchies package to run the examples against the local node.

If running the examples on `testnet`, use the appropriate package IDs for the testnet deployment.

In case of running the examples against an existing network, this network needs to have a faucet to fund your accounts (the IOTA testnet (`https://api.testnet.iota.cafe`) supports this), and you need to specify this via environment variables.

## Environment Variables

You'll need one or more of the following environment variables depending on your setup:

| Name                    | Required for local node | Required for testnet | Required for other node |
| ----------------------- | :---------------------: | :------------------: | :---------------------: |
| IOTA_HIERARCHIES_PKG_ID |            x            |          x           |            x            |
| API_ENDPOINT            |                         |          x           |            x            |

## Setup

Install dependencies:

```bash
npm install
```

Build the examples:

```bash
npm run build
```

## Running Examples

Run a specific example using:

```bash
npm run example <example-name>
```

For instance, to run the `01_create_federation` example:

```bash
IOTA_HIERARCHIES_PKG_ID=0x... npm run example 01_create_federation
```

Run all examples in sequence:

```bash
IOTA_HIERARCHIES_PKG_ID=0x... npm run example all
```

## Basic Examples

The following examples demonstrate the core hierarchies workflow in TypeScript:

| Name                                                                              | Information                                                                |
| :-------------------------------------------------------------------------------- | :------------------------------------------------------------------------- |
| [01_create_federation](src/01_create_federation.ts)                               | Demonstrates how to create a new federation as the root authority.         |
| [02_add_root_authority](src/02_add_root_authority.ts)                             | Shows how to add additional root authorities to a federation.              |
| [03_add_statement](src/03_add_statement.ts)                                       | Demonstrates adding trusted statements/properties to a federation.         |
| [04_create_accreditation_to_attest](src/04_create_accreditation_to_attest.ts)     | Shows how to grant attestation rights to entities for specific statements. |
| [05_revoke_accreditation_to_attest](src/05_revoke_accreditation_to_attest.ts)     | Demonstrates revoking attestation rights from entities.                    |
| [06_create_accreditation_to_accredit](src/06_create_accreditation_to_accredit.ts) | Shows how to delegate accreditation rights to other entities.              |
| [07_revoke_accreditation_to_accredit](src/07_revoke_accreditation_to_accredit.ts) | Demonstrates revoking accreditation rights from entities.                  |
| [08_revoke_root_authority](src/08_revoke_root_authority.ts)                       | Shows how to revoke root authority status from an entity.                  |
| [09_reinstate_root_authority](src/09_reinstate_root_authority.ts)                 | Demonstrates reinstating a previously revoked root authority.              |

## Validation Examples

The validation examples show how to verify trust relationships and validate statements in TypeScript:

| Name                                                               | Information                                                                  |
| :----------------------------------------------------------------- | :--------------------------------------------------------------------------- |
| [01_get_accreditations](src/validation/01_get_accreditations.ts)   | Demonstrates retrieving attestation and accreditation data from federations. |
| [02_validate_statements](src/validation/02_validate_statements.ts) | Shows how to validate if an entity can attest to specific statements.        |
| [03_get_statements](src/validation/03_get_statements.ts)           | Demonstrates retrieving statements from federations.                         |

## Utility Functions

| Name                   | Information                                                                   |
| :--------------------- | :---------------------------------------------------------------------------- |
| [util.ts](src/util.ts) | Common utility functions for setting up clients and handling federation data. |
| [main.ts](src/main.ts) | Main entry point that orchestrates all examples and provides CLI interface.   |

## Key Features of WASM Bindings

### Browser Compatibility

- Works in modern web browsers with WebAssembly support
- Supports both Node.js and browser environments
- Efficient memory management for large-scale applications

### TypeScript Support

- Full TypeScript definitions included
- Strong typing for all hierarchies operations
- IntelliSense support in compatible editors

### Performance Benefits

- Near-native performance through WebAssembly
- Optimized for client-side validation operations
- Minimal overhead for trust hierarchy operations

## Usage Patterns

### Web Application Integration

```typescript
import {
    HierarchiesClient,
    HierarchiesClientReadOnly,
} from "@iota/hierarchies/web";
import { IotaClient } from "@iota/iota-sdk/client";

// Initialize the IOTA client
const iotaClient = new IotaClient({ url: "https://api.testnet.iota.cafe" });
const hierarchies = await HierarchiesClientReadOnly.createWithPkgId(
    iotaClient,
    "0x...",
);

// Create a new federation
const federation = await hierarchies.createNewFederation()
    .buildAndExecute(hierarchies);
```

### Node.js Application Integration

```typescript
import {
    HierarchiesClient,
    HierarchiesClientReadOnly,
} from "@iota/hierarchies/node";
import { IotaClient } from "@iota/iota-sdk/client";

// Initialize client for Node.js environment
const iotaClient = new IotaClient({ url: "https://api.testnet.iota.cafe" });
const hierarchies = await HierarchiesClientReadOnly.createWithPkgId(
    iotaClient,
    "0x...",
);

// Validate statements
const isValid = await hierarchies.validateStatements(
    attesterId,
    statements,
);
```

## Example Workflow

### Complete Trust Hierarchy Setup

1. **Create Federation** (`01_create_federation`)
   - Establish root authority for your organization
   - Set up basic federation parameters

2. **Add Root Authorities** (`02_add_root_authority`)
   - Delegate governance to multiple entities
   - Distribute control across trusted parties

3. **Define Statements** (`03_add_statement`)
   - Create trusted properties/credentials
   - Set validation rules and constraints

4. **Grant Attestation Rights** (`04_create_accreditation_to_attest`)
   - Allow entities to provide attestations
   - Control who can validate specific statements

5. **Delegate Authority** (`06_create_accreditation_to_accredit`)
   - Enable hierarchical trust delegation
   - Create multi-level authorization chains

6. **Validate Operations** (`validation/*`)
   - Verify trust relationships
   - Validate statement attestations

### Lifecycle Management

- **Revocation** (`05_revoke_*`, `07_revoke_*`, `08_revoke_*`)
  - Temporarily or permanently revoke authorities
  - Maintain security through controlled access

- **Reinstatement** (`09_reinstate_root_authority`)
  - Restore previously revoked authorities
  - Handle authority lifecycle management

## Best Practices for Web Development

1. **Environment Detection**: Check for WebAssembly support before initialization
2. **Async Operations**: Always handle hierarchies operations asynchronously
3. **Error Boundaries**: Implement proper error handling in React/Vue components
4. **Memory Management**: Dispose of large objects when no longer needed
5. **Network Optimization**: Cache federation data when possible

## Security Considerations for Web Apps

- Store private keys securely (never in plain text)
- Validate all user inputs before creating statements
- Use HTTPS for all network communications
- Implement proper CORS policies for API access
- Consider using Web Workers for intensive operations

For more detailed information about IOTA Hierarchies WASM bindings and advanced usage patterns, refer to the official IOTA documentation.
