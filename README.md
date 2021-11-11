# Substrate Runtime and Contract Interactions

A Substrate node demonstrating interactions between the runtime and Ink! smart contracts.

## Background

### Contract-to-Runtime Interactions

The project demonstrates contract-to-runtime interactions through the use of [Chain Extensions](https://paritytech.github.io/ink-docs/macros-attributes/chain-extension/). Chain Extensions allow the exposure of runtime functions to smart contracts. The project uses Chain Extensions to make available a pallet function to smart contracts. See also the `rand-extension` chain extension code example, which is one example that this project _extended_. As in the example, this is enabled through a `chain_extension` created in the contract, and a `ChainExtension` impl defined in the runtime.

### Runtime-to-Contract Interactions

Runtime-to-contract interactions are enabled through invocations of the contracts pallet's own `bare_call` method, invoked from a custom pallet extrinsic.

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
