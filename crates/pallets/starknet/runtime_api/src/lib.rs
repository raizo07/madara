//! Definition of the runtime API for the Starknet pallet.

#![cfg_attr(not(feature = "std"), no_std)]
// Adding allow unused type parameters to avoid clippy errors
// generated by the `decl_runtime_apis` macro.
// Specifically, the macro generates a trait (`StarknetRuntimeApi`) with unused type parameters.
#![allow(clippy::extra_unused_type_parameters)]

use alloc::sync::Arc;

use blockifier::execution::contract_class::ContractClass;
use blockifier::transaction::objects::TransactionExecutionInfo;
use mp_felt::Felt252Wrapper;
use mp_transactions::{HandleL1MessageTransaction, Transaction, UserTransaction};
use sp_api::BlockT;
pub extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use mp_simulations::{PlaceHolderErrorTypeForFailedStarknetExecution, SimulationFlags};
use sp_runtime::DispatchError;
use starknet_api::api_core::{ChainId, ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Calldata, Event as StarknetEvent, Fee, MessageToL1, TransactionHash};

#[derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
pub enum StarknetTransactionExecutionError {
    ContractNotFound,
    ClassAlreadyDeclared,
    ClassHashNotFound,
    InvalidContractClass,
    ContractError,
}

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        /// Returns the nonce associated with the given address in the given block
        fn nonce(contract_address: ContractAddress) -> Nonce;
        /// Returns a storage slot value
        fn get_storage_at(address: ContractAddress, key: StorageKey) -> Result<StarkFelt, DispatchError>;
        /// Returns a `Call` response.
        fn call(address: ContractAddress, function_selector: EntryPointSelector, calldata: Calldata) -> Result<Vec<Felt252Wrapper>, DispatchError>;
        /// Returns the contract class hash at the given address.
        fn contract_class_hash_by_address(address: ContractAddress) -> ClassHash;
        /// Returns the contract class for the given class hash.
        fn contract_class_by_class_hash(class_hash: ClassHash) -> Option<ContractClass>;
        /// Returns the chain id.
        fn chain_id() -> Felt252Wrapper;
        /// Returns the Starknet OS Cairo program hash.
        fn program_hash() -> Felt252Wrapper;
        /// Returns the Starknet config hash.
        fn config_hash() -> StarkHash;
        /// Returns the fee token address.
        fn fee_token_address() -> ContractAddress;
        /// Returns fee estimate
        fn estimate_fee(transactions: Vec<UserTransaction>) -> Result<Vec<(u64, u64)>, DispatchError>;
        /// Returns message fee estimate
        fn estimate_message_fee(message: HandleL1MessageTransaction) -> Result<(u64, u64, u64), DispatchError>;
        /// Simulates transactions and returns their trace
        fn simulate_transactions(transactions: Vec<UserTransaction>, simulation_flags: SimulationFlags) -> Result<Vec<Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>>, DispatchError>;
        /// Filters extrinsic transactions to return only Starknet transactions
        ///
        /// To support runtime upgrades, the client must be unaware of the specific extrinsic
        /// details. To achieve this, the client uses an OpaqueExtrinsic type to represent and
        /// manipulate extrinsics. However, the client cannot decode and filter extrinsics due to
        /// this limitation. The solution is to offload decoding and filtering to the RuntimeApi in
        /// the runtime itself, accomplished through the extrinsic_filter method. This enables the
        /// client to operate seamlessly while abstracting the extrinsic complexity.
        fn extrinsic_filter(xts: Vec<<Block as BlockT>::Extrinsic>) -> Vec<Transaction>;
        fn get_index_and_tx_for_tx_hash(xts: Vec<<Block as BlockT>::Extrinsic>, chain_id: Felt252Wrapper, tx_hash: Felt252Wrapper) -> Option<(u32, Transaction)>;
        /// Returns events, call with index from get_index_and_tx_for_tx_hash method
        fn get_events_for_tx_by_index(tx_index: u32) -> Option<Vec<StarknetEvent>>;

        /// Return the list of StarknetEvent evmitted during this block, along with the hash of the starknet transaction they bellong to
        ///
        /// `block_extrinsics` is the list of all the extrinsic executed during this block, it is used in order to match
        fn get_starknet_events_and_their_associated_tx_hash(block_extrinsics: Vec<<Block as BlockT>::Extrinsic>, chain_id: Felt252Wrapper) -> Vec<(Felt252Wrapper, StarknetEvent)>;
        /// Return the outcome of the tx execution
        fn get_tx_execution_outcome(tx_hash: TransactionHash) -> Option<Vec<u8>>;
        /// Return the block context
        fn get_block_context() -> BlockContext;
        /// Return is fee disabled in state
        fn is_transaction_fee_disabled() -> bool;
        /// Return messages sent to L1 during tx execution
        fn get_tx_messages_to_l1(tx_hash: TransactionHash) -> Vec<MessageToL1>;
        /// Check if L1 Message Nonce has not been used
        fn l1_nonce_unused(nonce: Nonce) -> bool;
    }

    pub trait ConvertTransactionRuntimeApi {
        /// Converts the transaction to an UncheckedExtrinsic for submission to the pool.
        fn convert_transaction(transaction: UserTransaction) -> <Block as BlockT>::Extrinsic;

        /// Converts the L1 Message transaction to an UncheckedExtrinsic for submission to the pool.
        fn convert_l1_transaction(transaction: HandleL1MessageTransaction, fee: Fee) -> <Block as BlockT>::Extrinsic;

        /// Converts the DispatchError to an understandable error for the client
        fn convert_error(error: DispatchError) -> StarknetTransactionExecutionError;
    }
}

#[derive(Clone, Debug, parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
pub struct BlockContext {
    pub chain_id: String,
    pub block_number: u64,
    pub block_timestamp: u64,

    // Fee-related.
    pub sequencer_address: ContractAddress,
    pub fee_token_address: ContractAddress,
    pub vm_resource_fee_cost: Vec<(String, sp_arithmetic::fixed_point::FixedU128)>,
    pub gas_price: u128, // In wei.

    // Limits.
    pub invoke_tx_max_n_steps: u32,
    pub validate_max_n_steps: u32,
    pub max_recursion_depth: u32,
}

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

impl From<BlockContext> for blockifier::block_context::BlockContext {
    fn from(value: BlockContext) -> Self {
        Self {
            chain_id: ChainId(value.chain_id),
            block_number: BlockNumber(value.block_number),
            block_timestamp: BlockTimestamp(value.block_timestamp),
            sequencer_address: value.sequencer_address,
            fee_token_address: value.fee_token_address,
            vm_resource_fee_cost: Arc::new(HashMap::from_iter(value.vm_resource_fee_cost)),
            gas_price: value.gas_price,
            invoke_tx_max_n_steps: value.invoke_tx_max_n_steps,
            validate_max_n_steps: value.validate_max_n_steps,
            max_recursion_depth: value.max_recursion_depth,
        }
    }
}

impl From<blockifier::block_context::BlockContext> for BlockContext {
    fn from(value: blockifier::block_context::BlockContext) -> Self {
        Self {
            chain_id: value.chain_id.0,
            block_number: value.block_number.0,
            block_timestamp: value.block_timestamp.0,
            sequencer_address: value.sequencer_address,
            fee_token_address: value.fee_token_address,
            vm_resource_fee_cost: Vec::from_iter(value.vm_resource_fee_cost.iter().map(|(k, v)| (k.clone(), *v))),
            gas_price: value.gas_price,
            invoke_tx_max_n_steps: value.invoke_tx_max_n_steps,
            validate_max_n_steps: value.validate_max_n_steps,
            max_recursion_depth: value.max_recursion_depth,
        }
    }
}
