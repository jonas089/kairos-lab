use casper_engine_test_support::{
    ExecuteRequestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash, bytesrepr::{Bytes, FromBytes, ToBytes}, contracts::NamedKeys, crypto::{PublicKey, SecretKey}, runtime_args, system::{handle_payment::ARG_TARGET, mint::ARG_ID}, Key, RuntimeArgs, U256
};
use std::path::Path;
use base64::{engine::general_purpose::STANDARD, Engine};
use casper_engine_test_support::{InMemoryWasmTestBuilder, PRODUCTION_RUN_GENESIS_REQUEST};
use casper_types::{ContractHash, URef};
use std::env;

pub const ADMIN_SECRET_KEY: [u8; 32] = [1u8; 32];
pub const USER_SECRET_KEY: [u8; 32] = [2u8; 32];

pub struct TestContext {
    builder: InMemoryWasmTestBuilder,
    pub admin: AccountHash,
    pub user: AccountHash,
    pub contract_hash: ContractHash,
    pub contract_package_key: Key,
}

impl TestContext {
    pub fn new() -> TestContext {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);
        let admin: AccountHash = create_funded_account_for_secret_key_bytes(&mut builder, ADMIN_SECRET_KEY);
        let user: AccountHash = create_funded_account_for_secret_key_bytes(&mut builder, USER_SECRET_KEY);
        let market_maker_path: std::path::PathBuf = std::path::Path::new(env!("PATH_TO_WASM_BINARIES"))
            .join("demo-contract-optimized.wasm");
        install_wasm_with_args(
            &mut builder,
            &market_maker_path,
            admin,
            runtime_args! {},
        );

        let contract_hash = builder
            .get_expected_account(admin)
            .named_keys()
            .get("kairos_contract_hash")
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");

        let contract = builder
            .get_contract(contract_hash)
            .expect("should have contract");
    
        let contract_package = builder
            .get_expected_account(admin)
            .named_keys()
            .get("kairos_contract_package_hash")
            .expect("must haveses contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");

        TestContext {
            builder,
            admin,
            user,
            contract_hash,
            contract_package_key: contract_package.into(),
        }
    }
    
    pub fn submit_batch(&mut self, payload: Vec<u8>, sender: AccountHash){
        let session_args: RuntimeArgs = runtime_args!{
            "risc0_receipt" => Bytes::from(payload)
        };
        let submit_batch_request = ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            self.contract_hash,
            "submit_batch",
            session_args
        ).build();

        self.builder
            .exec(submit_batch_request)
            .commit()
            .expect_success();
    }
}

pub fn install_wasm_with_args(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    session_wasm_path: &Path,
    user: AccountHash,
    runtime_args: RuntimeArgs,
) {
    let session_request =
        ExecuteRequestBuilder::standard(user, session_wasm_path.to_str().unwrap(), runtime_args)
            .build();
    builder
        .exec(session_request)
        .expect_success()
        .commit();
}

pub fn create_funded_account_for_secret_key_bytes(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    account_secret_key_bytes: [u8; 32],
) -> AccountHash {
    let account_secret_key = SecretKey::ed25519_from_bytes(account_secret_key_bytes).unwrap();
    let account_public_key = PublicKey::from(&account_secret_key);
    let account_hash = account_public_key.to_account_hash();
    let transfer = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            ARG_AMOUNT => DEFAULT_ACCOUNT_INITIAL_BALANCE / 10_u64,
            ARG_TARGET => account_hash,
            ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer).expect_success().commit();
    account_hash
}

fn make_dictionary_item_key(admin: Key) -> String {
    let preimage = admin.to_bytes().unwrap();
    STANDARD.encode(preimage)
}

// Creates a dummy account and transfer funds to it
pub fn create_funded_dummy_account(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    account_string: Option<[u8; 32]>,
) -> AccountHash {
    let (_, account_public_key) =
        create_dummy_key_pair(if let Some(account_string) = account_string {
            account_string
        } else {
            [7u8; 32]
        });
    let account = account_public_key.to_account_hash();
    fund_account(builder, account);
    account
}

pub fn create_dummy_key_pair(account_string: [u8; 32]) -> (SecretKey, PublicKey) {
    let secret_key =
        SecretKey::ed25519_from_bytes(account_string).expect("failed to create secret key");
    let public_key = PublicKey::from(&secret_key);
    (secret_key, public_key)
}

pub fn fund_account(builder: &mut WasmTestBuilder<InMemoryGlobalState>, account: AccountHash) {
    let transfer = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            ARG_AMOUNT => DEFAULT_ACCOUNT_INITIAL_BALANCE / 10_u64,
            ARG_TARGET => account,
            ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer).expect_success().commit();
}