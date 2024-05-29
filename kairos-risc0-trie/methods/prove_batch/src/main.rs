#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std] // std support is experimental

use kairos_trie_integrations::ProofInputs;
use risc0_zkvm::guest::env;
use serde_json_wasm::to_vec;
risc0_zkvm::guest::entry!(main);
extern crate alloc;
use alloc::vec::Vec;

fn main() {
    let proof_inputs: ProofInputs = env::read();

    let output: Vec<u8> = to_vec(&proof_inputs.run_batch_proof_logic().unwrap()).unwrap();

    env::commit_slice(&output);
}