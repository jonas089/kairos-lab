#!/bin/bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
wasm-opt --strip-debug --signext-lowering ./target/wasm32-unknown-unknown/release/demo-contract.wasm -o ./binaries/demo-contract-optimized.wasm