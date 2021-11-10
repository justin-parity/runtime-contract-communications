# Substrate Runtime and Contract Interactions

A Substrate node demonstrating communication two ways, between runtime and Ink! smart contracts.

## Motivation

## Background

There are two mechanisms demonstrated in this project, to enable the communication between the runtime and smart contracts

### Contract-to-Runtime Communicaton

The project demonstrates contract-to-runtime interactions through the use of [Chain Extensions](https://paritytech.github.io/ink-docs/macros-attributes/chain-extension/). Chain Extensions allow the exposure of runtime functions to smart contracts. The project uses Chain Extensions to make available a pallet function to smart contracts. See also the `rand-extension` chain extension code example, which is one example that this project _extended_.

#### Implementation

Implementing the chain extension requires changes in the runtime, as well as any smart contracts that wish to extend runtime functions.

### Runtime-to-Contract Communication

## Getting Started

See 'Prepare a Substrate node using the Node Template' on the [Create your first Substrate blockchain](https://docs.substrate.io/tutorials/v3/create-your-first-substrate-chain/) page.

### Run

Use Rust's native `cargo` command to build and launch the template node:

```sh
cargo run --release -- --dev --tmp
```

### Build

The `cargo run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
cargo build --release
```
