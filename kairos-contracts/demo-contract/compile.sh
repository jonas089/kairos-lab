#!/bin/bash
rm -rf ../binaries/*
rustup target add wasm32-unknown-unknown
cd contract && cargo build --release --target wasm32-unknown-unknown
cd ../deposit-session && cargo build --release --target wasm32-unknown-unknown && cd ../
wasm-opt --strip-debug --signext-lowering ../target/wasm32-unknown-unknown/release/demo-contract.wasm -o ../binaries/demo-contract-optimized.wasm
wasm-opt --strip-debug --signext-lowering ../target/wasm32-unknown-unknown/release/deposit-session.wasm -o ../binaries/deposit-session-optimized.wasm