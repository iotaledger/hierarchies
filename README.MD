<div id="top"></div>

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->

<!-- PROJECT LOGO -->
<br />
<div align="center">
    <a href="https://github.com/iotaledger/template">
        <img src="banner.png" alt="Banner">
    </a>
    <h3 align="center">ITH</h3>
    <p align="center">
        IOTA Trust Hierarchies
        <br />
        <a href="https://wiki.iota.org"><strong>Explore the docs »</strong></a>
        <br />
        <br />
        <a href="https://github.com/iotaledger/template/labels/bug">Report Bug</a>
        ·
        <a href="https://github.com/iotaledger/template/labels/request">Request Feature</a>
    </p>
</div>
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#ith-notary-module">About The Project</a>
    </li>
    <li>
      <a href="#overview">Overview</a>
    </li>
    <li>
      <a href="#key-components">Key components</a>
      <ul>
        <li><a href="#federation">Federation</a></li>
        <li><a href="#root-authority">Root Authority</a></li>
        <li><a href="#governance">Governance</a></li>
        <li><a href="#credentials">Credentials</a></li>
        <li><a href="#permissions">Permissions</a></li>
        <li><a href="#trusted-property-constraints">Trusted Property Constraints</a></li>
      </ul>
    </li>
    <li>
      <a href="#workflow">Workflow</a>
    </li>
    <li><a href="#sequence-diagram">Sequence Diagram</a></li>
    <li><a href="#class-diagram">Class Diagram</a></li>
    <li><a href="#functions">API - Functions</a></li>
    <li><a href="#usage">Usage</a></li>
  </ol>
</details>

ITH Notary Module

The ITH Notary module is designed to manage a hierarchical trust system within a federation. It includes structures and methods for creating federations, issuing credentials, and managing permissions to attest and accredit trusted properties. This module leverages the Move programming language and the Sui blockchain.

## Overview

The ITH Notary module provides a framework for establishing and managing trust within a federation of entities. It includes mechanisms for:

1. **Creating Federations**: A federation is the primary structure representing a group of entities with a shared trust framework.
2. **Issuing Credentials**: Credentials are issued to entities within the federation, attesting to certain trusted properties.
3. **Managing Permissions**: Permissions to attest and accredit are managed to control who can issue credentials and what properties they can attest to.
4. **Governance**: A governance structure manages the rules and constraints for trusted properties and credentials.

## Key Components

### Federation

A `Federation` represents the hierarchy of trust in the system. It is a shared object that includes a set of root authorities and a governance structure.

### Root Authority

Root authorities are entities with the highest level of trust within a federation. They can delegate trust to other entities and manage permissions.

### Governance

The governance structure within a federation manages the trusted property constraints, accreditors, attesters, and the state of issued credentials. It ensures that only authorized entities can issue and attest credentials.

### Credentials

Credentials are issued to entities within the federation, attesting to specific trusted properties. Each credential has a validity period and can be revoked if necessary.

### Permissions

Permissions control the ability to attest and accredit trusted properties. There are two main types of permissions:

- **Permissions to Accredit**: Allow entities to delegate the ability to attest to other entities.
- **Permissions to Attest**: Allow entities to attest to certain trusted properties.

### Trusted Property Constraints

Trusted property constraints define the rules and allowed values for properties that can be included in credentials. These constraints ensure that credentials issued within the federation adhere to the federation's governance rules.

## Workflow

1. **Create a Federation**:
   - Use the `new_federation` function to create a new federation.
   - The creator is granted the root authority capability.

2. **Add Root Authorities**:
   - Root authorities are added to the federation using the `add_root_authority` function.
   - Root authorities manage trust delegation within the federation.

3. **Define Trusted Properties**:
   - Define trusted properties and their constraints using the `add_trusted_property` function.
   - These properties are used in credentials issued within the federation.

4. **Issue Permissions**:
   - Issue permissions to entities within the federation to allow them to accredit and attest trusted properties.
   - Use the `create_accreditation` and `create_attestation` functions.

5. **Issue Credentials**:
   - Use the issued permissions to create credentials for entities.
   - Use the `issue_credential` function to issue credentials with specified trusted properties and validity periods.

6. **Manage Credentials**:
   - Revoke or update credentials as needed based on the federation's governance rules.
   - Use the `revoke_attestation` and `revoke_accreditation` functions to manage permissions.

## Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant Federation
    participant RootAuthority
    participant Governance

    User->>+Federation: new_federation()
    Federation-->>-User: FederationCreatedEvent, RootAuthorityCap,AttestCap,AccreditCap

    User->>+RootAuthority: add_root_authority(Federation, RootAuthorityCap, account_id)
    RootAuthority-->>-Federation: RootAuthority added

    User->>+Federation: add_trusted_property(RootAuthorityCap, property_name, allowed_values, allow_any)
    Federation->>Governance: Add TrustedPropertyConstraint

    User->>+Federation: revoke_trusted_property(RootAuthorityCap, property_name, valid_to_ms)
    Federation->>Governance: Remove TrustedPropertyConstraint

    User->>+Federation: create_accreditation(AccreditCap, receiver, want_property_constraints)
    Federation->>Governance: Check PermissionsToAccredit
    Governance-->>Federation: Permissions validated
    Federation-->>User: PermissionsToAccredit issued

    User->>+Federation: create_attestation(AttestCap, receiver, wanted_constraints)
    Federation->>Governance: Check PermissionsToAttest
    Governance-->>Federation: Permissions validated
    Federation-->>User: PermissionsToAttest issued

    User->>+Federation: revoke_attestation(AttestCap, user_id, permission_id)
    Federation->>Governance: Check Permissions
    Governance-->>Federation: Permissions validated
    Federation-->>User: PermissionToAttest revoked

    User->>+Federation: revoke_accreditation(AccreditCap, user_id, permission_id)
    Federation->>Governance: Check Permissions
    Governance-->>Federation: Permissions validated
    Federation-->>User: PermissionToAccredit revoked

```

### Class diagram

```mermaid
classDiagram
    class Federation {
        +UID id
        +Governance governance
        +vector~RootAuthority~ root_authorities
    }

    class RootAuthority {
        +UID id
        +String account_id
        +Permissions permissions
    }

    class Governance {
        +UID id
        +TrustedPropertyConstraints trusted_constraints
        +VecMap~ID, Permissions~ accreditors
        +VecMap~ID, Permissions~ attesters
    }

    class RootAuthorityCap {
        +UID id
        +ID federation_id
    }

    class AttestCap {
        +UID id
        +ID federation_id
    }

    class AccreditCap {
        +UID id
        +ID federation_id
    }

    class Event {
        +D data
    }

    class FederationCreatedEvent {
        +address federation_address
    }


    class Permissions {
        +vector~Permission~ permissions
    }

    class Permission {
      +UID id
      +ID federation_id
      +String created_by
      +VecMap~TrustedPropertyName, TrustedPropertyConstraint~ constraints
    }


    class TrustedPropertyConstraints {
        +VecMap~TrustedPropertyName, TrustedPropertyConstraint~ data
    }

    class TrustedPropertyConstraint {
        +TrustedPropertyName property_name
        +VecSet~TrustedPropertyValue~ allowed_values
        +Option~TrustedPropertyExpression~ expression
        +bool allow_any
        +Timespan timespan
    }

    class TrustedPropertyName {
        +vector names
    }

    class TrustedPropertyValue {
        +Option~String text
        +Option~u64 number
    }

    class Timespan {
      +Option~u64 valid_from_ms
      +Option~u64 valid_until_ms
    }

    Federation --> Governance
    Federation --> RootAuthority
    Governance --> TrustedPropertyConstraints
    Governance --> Permissions
    TrustedPropertyConstraints --> TrustedPropertyConstraint
    Permissions --> Permission
    TrustedPropertyConstraint --> TrustedPropertyName
    TrustedPropertyConstraint --> TrustedPropertyValue
    TrustedPropertyConstraint --> Timespan

```

## Functions

### new_federation

```rust
public fun new_federation(ctx: &mut TxContext)
```

Creates a new federation with an initial root authority. Emits a `FederationCreatedEvent` and transfers the root authority capability to the sender.

### add_trusted_property

```rust
public fun add_trusted_property(
  self: &mut Federation,
  cap: &RootAuthorityCap,
  property_name: TrustedPropertyName,
  allowed_values: VecSet<TrustedPropertyValue>,
  allow_any: bool,
  _ctx: &mut TxContext
)
```

Adds a trusted property to the federation with specified constraints. Only callable by a root authority.

### revoke_trusted_property

````rust
public fun revoke_trusted_property(
  self: &mut Federation,
  cap: &RootAuthorityCap,
  property_name: TrustedPropertyName,
  valid_to_ms: u64,
  ctx: &mut TxContext
)
```
Revokes a trusted property from the federation. Only callable by a root authority.

### add_root_authority

```rust
  public fun add_root_authority(
      self : &mut Federation,
      cap : &RootAuthorityCap,
      account_id : ID,
      ctx : &mut TxContext,
    )
```

Adds a another Root Authority

### create_accreditation

```rust
public fun create_accreditation(
  self: &mut Federation,
  cap: &AccreditCap,
  receiver: ID,
  want_property_constraints: vector<TrustedPropertyConstraint>,
  ctx: &mut TxContext
)
```

Issues a permission to accredit specified trusted properties. Ensures the issuer has the necessary permissions to accredit.

### create_attestation

```rust
public fun create_attestation(
  self: &mut Federation,
  cap: &AttestCap,
  receiver: ID,
  wanted_constraints: vector<TrustedPropertyConstraint>,
  ctx: &mut TxContext
)
```

Issues a permission to attest specified trusted properties. Ensures the issuer has the necessary permissions to attest.

### revoke_attestation

```rust
public fun revoke_attestation(
  self: &mut Federation,
  cap: &AttestCap,
  user_id: &ID,
  permission_id: &ID,
  ctx: &mut TxContext
)
```

Revokes a permission to attest from a specified user. Ensures the issuer has the necessary permissions to revoke.

### revoke_accreditation

```rust
public fun revoke_accreditation(
  self: &mut Federation,
  cap: &AccreditCap,
  user_id: &ID,
  permission_id: &ID,
  ctx: &mut TxContext
)
```

Revokes a permission to accredit from a specified user. Ensures the issuer has the necessary permissions to revoke.

## Usage

To use the ITH Notary module:

1. **Create a Federation**: Use the `new_federation` function to create a new federation.
2. **Add Root Authorities**: Add root authorities to the federation, which will manage the delegation of trust.
3. **Define Trusted Properties**: Define the trusted properties and constraints that will be used in credentials.
4. **Issue Permissions**: Issue permissions to entities within the federation to allow them to accredit and attest to trusted properties.
5. **Issue Credentials**: Use the issued permissions to create and validate credentials for entities within the federation.
6. **Manage Credentials**: Revoke or update credentials as needed based on the federation's governance rules.

The ITH Notary module provides a robust framework for managing hierarchical trust within a federation, ensuring that only authorized entities can issue and attest to trusted properties.
