mod utils;
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_types::{
    account::AccountHash, runtime_args, Key,
    URef, U512, contracts::NamedKeys, ContractHash, RuntimeArgs
};
use utils::create_funded_dummy_account;

pub const ACCOUNT_USER_1: [u8; 32] = [1u8; 32];
pub const ACCOUNT_USER_2: [u8; 32] = [2u8; 32];
pub const ACCOUNT_USER_3: [u8; 32] = [3u8; 32];

#[cfg(test)]
pub struct TestContext {
    pub builder: InMemoryWasmTestBuilder,
    pub account_1: AccountHash,
    pub account_2: AccountHash,
    pub account_3: AccountHash,
}

impl TestContext {
    pub fn new() -> TestContext {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);
        let account_1 = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));
        let account_2 = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_2));
        let account_3 = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_3));

        TestContext {
            builder,
            account_1,
            account_2,
            account_3,
        }
    }

    pub fn named_keys(&self, account: AccountHash) -> NamedKeys {
        self.builder
            .get_expected_account(account)
            .named_keys()
            .clone()
    }

    pub fn contract_named_keys(&self, contract_name: &str, key_name: &str, account: AccountHash) -> Key {
        let contract_hash = self.contract_hash_from_named_keys(contract_name, account);
        *self
            .builder
            .get_contract(contract_hash)
            .expect("should have contract")
            .named_keys()
            .get(key_name)
            .unwrap()
    }

    pub fn contract_hash_from_named_keys(&self, key_name: &str, account: AccountHash) -> ContractHash {
        self.named_keys(account)
            .get(key_name)
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash")
    }
    
    pub fn contract_hash(&self, name: &str, account: AccountHash) -> ContractHash{
        self.builder.get_expected_account(account)
            .named_keys()
            .get(name)
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash")
    }

    pub fn install(&mut self, admin: AccountHash) {
        let session_args = runtime_args! {};
        let install_contract_request =
            ExecuteRequestBuilder::standard(admin, "/Users/chef/Desktop/contracts/deposit-contracts/target/wasm32-unknown-unknown/release/deposit-contract-optimized.wasm", session_args)
            .build();
        self.builder
            .exec(install_contract_request)
            .expect_success()
            .commit();
    }

    pub fn create_contract_purse(&mut self, msg_sender: AccountHash, account: AccountHash) {
        let contract_hash = self.contract_hash("kairos_deposit_contract", account);
        let session_args = runtime_args! {};
        let create_contract_purse_request = ExecuteRequestBuilder::contract_call_by_hash(
            msg_sender,
            contract_hash,
            "create_purse",
            session_args,
        )
        .build();
        self.builder
            .exec(create_contract_purse_request)
            .expect_success()
            .commit();
    }

    pub fn get_contract_purse_uref(&self, account: AccountHash) -> URef {
        let seed_uref: URef = *self
            .contract_named_keys("kairos_deposit_contract", "kairos_deposit_purse", account)
            .as_uref()
            .unwrap();
        seed_uref
    }

    #[allow(dead_code)]
    pub fn get_contract_purse_balance(&self, account: AccountHash) -> U512 {
        let seed_uref: URef = *self
            .contract_named_keys("kairos_deposit_contract", "kairos_deposit_purse", account)
            .as_uref()
            .unwrap();
        let purse_balance = self.builder.get_purse_balance(seed_uref);
        purse_balance
    }

    pub fn run_deposit_session(&mut self, amount: U512, account: AccountHash) {
        let session_args = runtime_args! {
            "amount" => amount,
            "deposit_contract" => self.contract_hash("kairos_deposit_contract", account)
        };
        let session_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            "/Users/chef/Desktop/contracts/deposit-contracts/target/wasm32-unknown-unknown/release/deposit-session-optimized.wasm",
            session_args,
        )
        .build();
        self.builder.exec(session_request).expect_success().commit();
    }

    pub fn run_malicious_session(
        &mut self,
        msg_sender: AccountHash,
        amount: U512,
        account: AccountHash,
    ) {
        /*
        let contract_hash: AddressableEntityHash =
            self.contract_hash("kairos_deposit_contract", account);
        */
        let session_args = runtime_args! {
            "amount" => amount,
            "deposit_contract" => self.contract_hash("kairos_deposit_contract", account)
        };
        let session_request = ExecuteRequestBuilder::standard(
            msg_sender,
            "/Users/chef/Desktop/contracts/deposit-contracts/target/wasm32-unknown-unknown/release/malicious-session-optimized.wasm",
            session_args,
        )
        .build();
        self.builder.exec(session_request).expect_failure().commit();
    }

    pub fn run_withdrawal_session(
        &mut self,
        msg_sender: AccountHash,
        amount: U512,
        account: AccountHash,
    ) {
        let session_args = runtime_args! {
            "amount" => amount,
            "deposit_contract" => self.contract_hash("kairos_deposit_contract", account)
        };
        let session_request = ExecuteRequestBuilder::standard(
            msg_sender,
            "/Users/chef/Desktop/contracts/deposit-contracts/target/wasm32-unknown-unknown/release/withdrawal-session-optimized.wasm",
            session_args,
        )
        .build();
        self.builder.exec(session_request).expect_success().commit();
    }

    pub fn run_malicious_withdrawal_session(
        &mut self,
        msg_sender: AccountHash,
        amount: U512,
        account: AccountHash,
    ) {
        let session_args = runtime_args! {
            "amount" => amount,
            "deposit_contract" => self.contract_hash("kairos_deposit_contract", account)
        };
        let session_request = ExecuteRequestBuilder::standard(
            msg_sender,
            "/Users/chef/Desktop/contracts/deposit-contracts/target/wasm32-unknown-unknown/release/withdrawal-session-optimized.wasm",
            session_args,
        )
        .build();
        self.builder.exec(session_request).expect_failure().commit();
    }

    pub fn run_malicious_reader_session(
        &mut self,
        msg_sender: AccountHash,
        amount: U512,
        account: AccountHash,
        deposit_purse_uref: URef,
    ) {
        let session_args = runtime_args! {
            "amount" => amount,
            "deposit_contract" => self.contract_hash("kairos_deposit_contract", account),
            "purse_uref" => deposit_purse_uref
        };
        let session_request = ExecuteRequestBuilder::standard(
            msg_sender,
            "/Users/chef/Desktop/contracts/deposit-contracts/target/wasm32-unknown-unknown/release/malicious-reader-optimized.wasm",
            session_args,
        )
        .build();
        self.builder.exec(session_request).expect_failure().commit();
    }
}
