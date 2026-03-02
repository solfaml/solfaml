# SolfaML Parser

The Solfa Markup Language parser, written in Rust.

## Setup

1. Install the Rust Toolchain:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Add WASM target and install wasm-pack:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Build

### WASM

To build the WASM library, run the following command:

```sh
wasm-pack build --features=wasm
# OR
# wasm-pack build --features=wasm --out-dir <path>
```
