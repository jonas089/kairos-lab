#![no_main]
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_event_standard::Schemas;
use casper_types::{
    contracts::NamedKeys, runtime_args, AccessRights, ApiError, CLValue, EntryPoints, Key,
    RuntimeArgs, URef, U512,
};
mod constants;
use constants::{
    KAIROS_CONTRACT_HASH, KAIROS_CONTRACT_PACKAGE_HASH, KAIROS_CONTRACT_UREF, KAIROS_DEPOSIT_PURSE,
    KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER, RUNTIME_ARG_AMOUNT, RUNTIME_ARG_TEMP_PURSE, RUNTIME_ARG_RECEIPT
};
mod entry_points;
mod utils;
use utils::errors::DepositError;
use utils::events::Deposit;
use utils::get_immediate_caller;
use risc0_zkvm::Receipt;
use serde_json_wasm::{from_slice};
use kairos_trie_integrations::ProofOutputs;

// This entry point is called once when the contract is installed.
// The contract purse will be created in contract context so that it is "owned" by the contract
// rather than the installing account.
#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(KAIROS_DEPOSIT_PURSE).is_some() {
        runtime::revert(DepositError::AlreadyInitialized);
    }
    // initialize event schema
    let schemas = Schemas::new().with::<Deposit>();
    casper_event_standard::init(schemas);
    let new_deposit_purse: URef = system::create_purse();
    runtime::put_key(KAIROS_DEPOSIT_PURSE, new_deposit_purse.into());
}

#[no_mangle]
pub extern "C" fn get_purse() {
    let deposit_purse: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(DepositError::MissingKeyDepositPurse)
        .into_uref()
        .unwrap_or_revert();
    let reference_to_deposit_purse_with_restricted_access =
        deposit_purse.with_access_rights(AccessRights::ADD);
    runtime::ret(
        CLValue::from_t(reference_to_deposit_purse_with_restricted_access)
            .unwrap_or_revert_with(DepositError::FailedToReturnContractPurseAsReference),
    );
}

// Entry point called by a user through session code to deposit funds.
// Due to Casper < 2.0 purse management and access control, it is necessary that
// a temporary purse is funded and passed to the contract, since this is
// the only secure method of making a payment to a contract purse.
#[no_mangle]
pub extern "C" fn deposit() {
    let temp_purse: URef = runtime::get_named_arg(RUNTIME_ARG_TEMP_PURSE);
    let amount: U512 = runtime::get_named_arg(RUNTIME_ARG_AMOUNT);
    let deposit_purse_uref: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(DepositError::MissingKeyDepositPurse)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    system::transfer_from_purse_to_purse(temp_purse, deposit_purse_uref, amount, None)
        .unwrap_or_revert();

    // kairos utilizes u64 so only amounts that can be converted are accepted.
    let amount =
        u64::try_from(amount).unwrap_or_else(|_| runtime::revert(ApiError::InvalidArgument));

    let new_deposit_record: Deposit = Deposit {
        depositor: get_immediate_caller().unwrap_or_revert(),
        amount,
    };
    // this increases a counter automatically - we don't need to create one ourselves
    casper_event_standard::emit(new_deposit_record);
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Proof {
    pub receipt: Receipt,
    pub program_id: [u32;8],
}

#[no_mangle]
pub extern "C" fn submit_batch(){
    let proof_serialized: Vec<u8> = runtime::get_named_arg(RUNTIME_ARG_RECEIPT);
    let proof: Proof = from_slice(&proof_serialized).unwrap();
    match proof.receipt.verify(proof.program_id){
        Ok(_) => {},
        // replace ApiError with meaningful UserError
        Err(_) => runtime::revert(ApiError::InvalidArgument)
    };
    let journal: ProofOutputs = serde_json_wasm::from_slice(&proof.receipt.journal.bytes).unwrap();
    // todo: update root
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = EntryPoints::from(vec![
        entry_points::init(),
        entry_points::get_purse(),
        entry_points::deposit(),
        entry_points::submit_batch()
    ]);

    // this counter will be udpated by the entry point that processes / verifies batches
    let last_processed_deposit_counter = storage::new_uref(0u64);
    let named_keys = NamedKeys::from([(
        KAIROS_LAST_PROCESSED_DEPOSIT_COUNTER.to_string(),
        last_processed_deposit_counter.into(),
    )]);

    let (contract_hash, _) = storage::new_locked_contract(
        entry_points,
        Some(named_keys),
        Some(KAIROS_CONTRACT_PACKAGE_HASH.to_string()),
        Some(KAIROS_CONTRACT_UREF.to_string()),
    );
    
    let contract_hash_key = Key::from(contract_hash);
    runtime::put_key(KAIROS_CONTRACT_HASH, contract_hash_key);

    // Call the init entry point of the newly installed contract
    // This will setup the deposit purse and initialize Event Schemas (CES)
    let init_args = runtime_args! {};
    runtime::call_contract::<()>(contract_hash, "init", init_args);
}
