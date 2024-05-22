use risc0_zkvm::Receipt;
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Proof {
    pub receipt: Receipt,
    pub program_id: [u32;8],
}

#[test]
fn generate_proof(){
    extern crate alloc;
    use alloc::rc::Rc;
    use avi_trie_integrations::{ProofInputs, transactions::*, account_trie::AccountTrie, account_trie::Account};
    use circuits::{PROVE_BATCH_ELF, PROVE_BATCH_ID};
    use kairos_trie::{TrieRoot, stored::memory_db::MemoryDb, DigestHasher};
    let alice_public_key = "alice_public_key".as_bytes().to_vec();
    let bob_public_key = "bob_public_key".as_bytes().to_vec();
    let batches = vec![
        vec![
            KairosTransaction::Deposit(Deposit {
                recipient: alice_public_key.clone(),
                amount: 10,
            }),
            KairosTransaction::Transfer(Signed {
                public_key: alice_public_key.clone(),
                transaction: Transfer {
                    recipient: bob_public_key.clone(),
                    amount: 5,
                },
                nonce: 0,
            }),
            KairosTransaction::Withdraw(Signed {
                public_key: alice_public_key.clone(),
                transaction: Withdraw { amount: 5 },
                nonce: 1,
            }),
        ],
    ];


    let db = Rc::new(MemoryDb::<Account>::empty());
    let mut prior_root_hash = TrieRoot::default();

    for batch in batches.into_iter() {
        // the Trie is constructed from the current state of the DB.
        // keep in mind that the Trie, other than DeltaTree, stores Accounts
        // the entire DB state is used to construct a Snapshot for each proof.
        let mut account_trie = AccountTrie::new_try_from_db(db.clone(), prior_root_hash)
            .expect("Failed to create account trie");
        account_trie
            .apply_batch(batch.iter().cloned())
            .expect("Failed to apply batch");

        let new_root_hash = account_trie
            .txn
            .commit(&mut DigestHasher::<sha2::Sha256>::default())
            .expect("Failed to commit transaction");

        let trie_snapshot = account_trie.txn.build_initial_snapshot();

        let proof_inputs = ProofInputs {
            transactions: batch.into_boxed_slice(),
            trie_snapshot,
        };

        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&proof_inputs)
            .map_err(|e| format!("Error in ExecutorEnv builder write: {e}")).unwrap()
            .build()
            .map_err(|e| format!("Error in ExecutorEnv builder build: {e}")).unwrap();

        let receipt = risc0_zkvm::default_prover()
            .prove(env, PROVE_BATCH_ELF)
            .map_err(|e| format!("Error in risc0_zkvm prove: {e}")).unwrap();

        receipt
            .verify(PROVE_BATCH_ID)
            .map_err(|e| format!("Error in risc0_zkvm verify: {e}")).unwrap();

        let proof: Proof = Proof {
            receipt: receipt,
            program_id: PROVE_BATCH_ID
        };

        println!("[OK] Proof! {:?} Bytes receipt", &bincode::serialize(&proof).unwrap().len());
    }
}