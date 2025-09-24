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
| [03_add_properties](src/03_add_properties.ts)                                     | Demonstrates adding properties to a federation.                            |
| [04_create_accreditation_to_attest](src/04_create_accreditation_to_attest.ts)     | Shows how to grant attestation rights to entities for specific properties. |
| [05_revoke_accreditation_to_attest](src/05_revoke_accreditation_to_attest.ts)     | Demonstrates revoking attestation rights from entities.                    |
| [06_create_accreditation_to_accredit](src/06_create_accreditation_to_accredit.ts) | Shows how to delegate accreditation rights to other entities.              |
| [07_revoke_accreditation_to_accredit](src/07_revoke_accreditation_to_accredit.ts) | Demonstrates revoking accreditation rights from entities.                  |
| [08_revoke_root_authority](src/08_revoke_root_authority.ts)                       | Shows how to revoke root authority status from an entity.                  |
| [09_reinstate_root_authority](src/09_reinstate_root_authority.ts)                 | Demonstrates reinstating a previously revoked root authority.              |

## Validation Examples

The validation examples show how to verify trust relationships and validate properties in TypeScript:

| Name                                                               | Information                                                                  |
| :----------------------------------------------------------------- | :--------------------------------------------------------------------------- |
| [01_get_accreditations](src/validation/01_get_accreditations.ts)   | Demonstrates retrieving attestation and accreditation data from federations. |
| [02_validate_properties](src/validation/02_validate_properties.ts) | Shows how to validate if an entity can attest to specific properties.        |
| [03_get_properties](src/validation/03_get_properties.ts)           | Demonstrates retrieving properties from federations.                         |

## Real-World Examples

These examples demonstrate practical applications of IOTA Hierarchies in real business scenarios with comprehensive TypeScript implementation:

| Name                                                             | Information                                                                |
| :--------------------------------------------------------------- | :------------------------------------------------------------------------- |
| [01_university_degrees](src/real-world/01_university_degrees.ts) | University degree verification system with multi-level academic hierarchy. |
| [02_supply_chain](src/real-world/02_supply_chain.ts)             | Supply chain quality certification system with international standards.    |

### University Degree Verification System

Demonstrates a comprehensive academic credential verification system optimized for web applications:

- University consortium federation management
- Multi-level hierarchy: University → Faculty → Registrar
- Academic properties: degrees, fields of study, GPAs, graduation years
- Cross-institutional credential recognition
- Degree revocation for academic misconduct
- Mobile app integration and QR code verification

**Web-Specific Features:**

- Browser-based credential verification
- Real-time validation without server round-trips
- QR code integration for mobile verification
- Progressive Web App (PWA) compatibility
- Cross-platform TypeScript implementation

### Supply Chain Quality Certification System

Demonstrates a global supply chain certification system designed for web dashboards and e-commerce integration:

- International standards consortium (ISO-style) federation
- Multi-regional structure with real-time status monitoring
- Comprehensive certifications with web dashboard integration
- Consumer-facing QR code verification
- E-commerce API integration examples
- Real-time certificate expiry tracking

**Web-Specific Features:**

- Interactive certification dashboard
- RESTful API integration examples
- Consumer-facing verification interfaces
- Real-time compliance monitoring
- Mobile-responsive design patterns
- WebAssembly performance optimization

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

// Validate attestations
const isValid = await hierarchies.validateProperties(
    attesterId,
    properties,
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

3. **Define Properties** (`03_add_properties`)
   - Create trusted properties/credentials
   - Set validation rules and constraints

4. **Grant Attestation Rights** (`04_create_accreditation_to_attest`)
   - Allow entities to provide attestations
   - Control who can validate specific properties

5. **Delegate Authority** (`06_create_accreditation_to_accredit`)
   - Enable hierarchical trust delegation
   - Create multi-level authorization chains

6. **Validate Operations** (`validation/*`)
   - Verify trust relationships
   - Validate attested properties

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
- Validate all user inputs before creating properties
- Use HTTPS for all network communications
- Implement proper CORS policies for API access
- Consider using Web Workers for intensive operations

For more detailed information about IOTA Hierarchies WASM bindings and advanced usage patterns, refer to the official IOTA documentation.
