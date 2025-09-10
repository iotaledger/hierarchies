![banner](https://github.com/iotaledger/hierarchies/raw/HEAD/.github/banner_hierarchies.png)

<p align="center">
  <a href="https://iota.stackexchange.com/" style="text-decoration:none;"><img src="https://img.shields.io/badge/StackExchange-9cf.svg?logo=stackexchange" alt="StackExchange"></a>
  <a href="https://discord.gg/iota-builders" style="text-decoration:none;"><img src="https://img.shields.io/badge/Discord-9cf.svg?logo=discord" alt="Discord"></a>
  <a href="https://github.com/iotaledger/hierarchies/blob/main/LICENSE" style="text-decoration:none;"><img src="https://img.shields.io/github/license/iotaledger/hierarchies.svg" alt="Apache 2.0 license"></a>
</p>

<p align="center">
  <a href="#introduction">Introduction</a> ◈
  <a href="#documentation-and-resources">Documentation & Resources</a> ◈
  <a href="#components">Components</a> ◈
  <a href="#contributing">Contributing</a>
</p>

---

# IOTA Hierarchies

## Introduction

IOTA Hierarchies enables the creation of structured, hierarchical trust networks on the IOTA ledger. It allows organizations to delegate authority and attestation rights across multiple levels, creating verifiable chains of trust for any arbitrary properties or credentials.

IOTA Hierarchies is composed of two primary components:

- **Hierarchies Move Package**: The on-chain smart contracts that define federations, accreditations, and attestations.
- **Hierarchies Library (Rust)**: A client-side library that provides developers with convenient functions to create, manage, and validate hierarchical trust structures.

## Documentation and Resources

- [Hierarchies Documentation Pages](https://docs.iota.org/developer/iota-hierarchies): Supplementing documentation with context around hierarchies and simple examples on library usage.
- API References:
  - [Rust API Reference](https://iotaledger.github.io/hierarchies/hierarchies/index.html): Package documentation.

- [Wasm API Reference](https://docs.iota.org/developer/iota-hierarchies/references/wasm/api_ref): Wasm Package documentation.

- Examples:
  - [Rust Examples](https://github.com/iotaledger/hierarchies/tree/main/hierarchies-rs/examples/README.md): Practical code snippets to get you started with the library in Rust.
  - [Wasm Examples](https://github.com/iotaledger/hierarchies/tree/main/bindings/wasm/hierarchies_wasm/examples/README.md): Practical code snippets to get you started with the library in TypeScript/JavaScript.

## Bindings

[Foreign Function Interface (FFI)](https://en.wikipedia.org/wiki/Foreign_function_interface) Bindings of this [Rust](https://www.rust-lang.org/) library to other programming languages:

- [Web Assembly](https://github.com/iotaledger/hierarchies/tree/main/bindings/wasm/hierarchies_wasm) (JavaScript/TypeScript)

## Contributing

We would love to have you help us with the development of IOTA Hierarchies. Each and every contribution is greatly valued!

Please review the [contribution](https://docs.iota.org/developer/iota-hierarchies/contribute) sections in the [IOTA Docs Portal](https://docs.iota.org/developer/iota-hierarchies/).

To contribute directly to the repository, simply fork the project, push your changes to your fork and create a pull request to get them included!

The best place to get involved in discussions about this library or to look for support at is the `#hierarchies` channel on the [IOTA Discord](https://discord.gg/iota-builders). You can also ask questions on our [Stack Exchange](https://iota.stackexchange.com/).
