# IOTA Hierarchies Examples

The following code examples demonstrate how to use IOTA Hierarchies for creating, managing, and interacting with hierarchical trust networks on the IOTA network.

## Prerequisites

Examples can be run against:

- A local IOTA node
- An existing network, e.g., the IOTA testnet

When setting up a local node, you'll need to publish a hierarchies package as described in the IOTA documentation. You'll also need to provide environment variables for your locally deployed hierarchies package to run the examples against the local node.

If running the examples on `testnet`, use the appropriate package IDs for the testnet deployment.

In case of running the examples against an existing network, this network needs to have a faucet to fund your accounts (the IOTA testnet (`https://api.testnet.iota.cafe`) supports this), and you need to specify this via `API_ENDPOINT`.

## Environment Variables

You'll need one or more of the following environment variables depending on your setup:

| Name                    | Required for local node | Required for testnet | Required for other node |
| ----------------------- | :---------------------: | :------------------: | :---------------------: |
| IOTA_HIERARCHIES_PKG_ID |            x            |          x           |            x            |
| API_ENDPOINT            |                         |          x           |            x            |

## Running Examples

Run an example using the following command (environment variables depend on your setup):

```bash
IOTA_HIERARCHIES_PKG_ID=0x... cargo run --example <example-name>
```

For instance, to run the `01_create_federation` example:

```bash
IOTA_HIERARCHIES_PKG_ID=0x... cargo run --release --example 01_create_federation
```

## Basic Examples

The following examples demonstrate the core hierarchies workflow:

| Name                                                                          | Information                                                                |
| :---------------------------------------------------------------------------- | :------------------------------------------------------------------------- |
| [01_create_federation](01_create_federation.rs)                               | Demonstrates how to create a new federation as the root authority.         |
| [02_add_root_authority](02_add_root_authority.rs)                             | Shows how to add additional root authorities to a federation.              |
| [03_add_properties](03_add_properties.rs)                                     | Demonstrates adding properties to a federation.                            |
| [04_create_accreditation_to_attest](04_create_accreditation_to_attest.rs)     | Shows how to grant attestation rights to entities for specific properties. |
| [05_revoke_accreditation_to_attest](05_revoke_accreditation_to_attest.rs)     | Demonstrates revoking attestation rights from entities.                    |
| [06_create_accreditation_to_accredit](06_create_accreditation_to_accredit.rs) | Shows how to delegate accreditation rights to other entities.              |
| [07_revoke_accreditation_to_accredit](07_revoke_accreditation_to_accredit.rs) | Demonstrates revoking accreditation rights from entities.                  |
| [08_revoke_root_authority](08_revoke_root_authority.rs)                       | Shows how to revoke root authority status from an entity.                  |
| [09_reinstate_root_authority](09_reinstate_root_authority.rs)                 | Demonstrates reinstating a previously revoked root authority.              |

## Validation Examples

The validation examples show how to verify trust relationships and validate properties:

| Name                                                                                           | Information                                                                  |
| :--------------------------------------------------------------------------------------------- | :--------------------------------------------------------------------------- |
| [01_get_accreditations](validation/01_get_attestations_and_accreditations.rs)                  | Demonstrates retrieving attestation and accreditation data from federations. |
| [02_validate_properties](validation/02_validate_properties.rs)                                 | Shows how to validate if an entity can attest to specific properties.        |
| [03_get_properties](validation/03_get_properties.rs)                                           | Demonstrates retrieving properties from federations.                         |

## Getting Started Example

| Name                                                  | Information                                                   |
| :---------------------------------------------------- | :------------------------------------------------------------ |
| [getting_started](getting_started/getting_started.rs) | Comprehensive walkthrough of the main hierarchies operations. |

## Key Concepts

### Federation Management

- **Federation Creation**: Establishing the root trust authority
- **Root Authority Management**: Adding, revoking, and reinstating root authorities
- **Statement Definition**: Defining trusted properties within the federation

### Trust Delegation

- **Accreditation to Attest**: Granting rights to provide attestations for specific properties
- **Accreditation to Accredit**: Delegating the ability to grant attestation rights to others
- **Hierarchical Structure**: Creating multi-level trust chains

### Validation and Verification

- **Properties Validation**: Verifying if entities can attest to specific properties
- **Trust Chain Verification**: Validating the complete chain of authority
- **On-chain vs Off-chain**: Choosing appropriate validation methods for performance

## Example Scenarios

### University Credential System

1. **University** creates federation for academic credentials
2. **Departments** receive accreditation rights for their domain
3. **Professors** receive attestation rights for specific courses
4. **External Verifiers** validate student credentials

### Supply Chain Verification

1. **Certification Body** creates federation for product standards
2. **Regional Offices** receive accreditation rights for their territories
3. **Local Inspectors** receive attestation rights for specific standards
4. **Consumers/Retailers** validate product compliance

## Error Handling

The examples demonstrate proper error handling for common scenarios:

- Attempting operations without proper authority
- Validating properties for revoked entities
- Network and transaction failures
- Invalid properties configurations

## Security Considerations

- All federation data is publicly readable on the blockchain
- Private keys control all hierarchies operations
- Revocation affects entire downstream trust chains
- Root authorities have ultimate control over federation governance

For more detailed information about IOTA Hierarchies concepts and advanced usage, refer to the official IOTA documentation.
