

use elrond_wasm::{H256, Address};

use crate::big_int_mock::*;
use crate::big_uint_mock::*;
use crate::display_util::*;

use elrond_wasm::ContractHookApi;
// use elrond_wasm::CallableContract;
use elrond_wasm::BigUintApi;
use elrond_wasm::err_msg;

use num_bigint::{BigInt, BigUint};
use num_traits::cast::ToPrimitive;

// use alloc::boxed::Box;
use alloc::vec::Vec;

use std::collections::HashMap;
use std::fmt;
// use std::fmt::Write;

use core::cell::RefCell;
use alloc::rc::Rc;

use sha3::{Sha3_256, Keccak256, Digest};

const ADDRESS_LENGTH: usize = 32;
const KEY_LENGTH: usize = 32;
const TOPIC_LENGTH: usize = 32;

#[derive(Clone)]
pub struct TxInput {
    pub from: Address,
    pub to: Address,
    pub call_value: BigUint,
    pub func_name: Vec<u8>,
    pub args: Vec<Vec<u8>>,
}

impl fmt::Display for TxInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TxInput {{ func: {}, args: {:?}, call_value: {}, from: 0x{}, to: 0x{}\n}}", 
            String::from_utf8(self.func_name.clone()).unwrap(), 
            self.args, 
            self.call_value,
            address_hex(&self.from), 
            address_hex(&self.to))
    }
}

impl TxInput {
    // pub fn new_create(new_contract: Vec<u8>, from: Address, to: Address) -> Self {
    //     TxInput{
    //         func_name: b"init".to_vec(),
    //         args: Vec::new(),
    //         call_value: 0u32.into(),
    //         from,
    //         to,
    //     }
    // }

    // pub fn new_call(func_name: &'static str, from: Address, to: Address) -> Self {
    //     TxInput{
    //         func_name: func_name.as_bytes().to_vec(),
    //         args: Vec::new(),
    //         call_value: 0u32.into(),
    //         from,
    //         to,
    //     }
    // }

    pub fn add_arg(&mut self, arg: Vec<u8>) {
        self.args.push(arg);
    }
}

#[derive(Clone)]
pub struct TxResult {
    pub result_status: i32,
    pub result_message: Vec<u8>,
    pub result_values: Vec<Vec<u8>>,
}

impl fmt::Display for TxResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let results_hex: Vec<String> = self.result_values.iter().map(|r| format!("0x{}", hex::encode(r))).collect();
        write!(f, "TxResult {{\n\tresult_status: {},\n\tresult_values:{:?}\n}}", self.result_status, results_hex)
    }
}

impl TxResult {
    pub fn empty() -> TxResult {
        TxResult {
            result_status: 0,
            result_message: Vec::new(),
            result_values: Vec::new(),
        }
    }
    pub fn print(&self) {
        println!("{}", self);
    }
}

pub struct TxMutableContext {
    pub contract_storage: HashMap<Vec<u8>, Vec<u8>>,
    pub result: TxResult,
}

pub struct TxContext {
    pub tx_input: TxInput,
    pub mut_context_cell: Rc<RefCell<TxMutableContext>>,
}

impl TxContext {
    pub fn new(tx_input: TxInput, mut_context: TxMutableContext) -> Self {
        TxContext {
            tx_input,
            mut_context_cell: Rc::new(RefCell::new(mut_context)),
        }
    }

    pub fn dummy() -> Self {
        TxContext {
            tx_input: TxInput{
                from: Address::zero(),
                to: Address::zero(),
                call_value: 0u32.into(),
                func_name: Vec::new(),
                args: Vec::new(),
            },
            mut_context_cell: Rc::new(RefCell::new(TxMutableContext{
                contract_storage: HashMap::new(),
                result: TxResult::empty(),
            })),
        }
    }
}

impl Clone for TxContext {
    fn clone(&self) -> Self {
        TxContext{
            tx_input: self.tx_input.clone(),
            mut_context_cell: Rc::clone(&self.mut_context_cell),
        }
    }
}

impl elrond_wasm::ContractHookApi<RustBigInt, RustBigUint> for TxContext {
    fn get_sc_address(&self) -> Address {
        self.tx_input.to.clone()
    }

    fn get_owner_address(&self) -> Address {
        self.get_caller() // TEMP !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        // let state = self.mut_context_cell.borrow();
        // match &state.current_tx {
        //     None => panic!("Tx not initialized!"),
        //     Some(tx) => {
        //         match state.accounts.get(&tx.to) {
        //             None => panic!("Account not found!"),
        //             Some(acct) => {
        //                 if let Some(contract_owner) = &acct.contract_owner {
        //                     contract_owner.clone()
        //                 } else {
        //                     panic!("Account is not a contract or does not have and owner specified")
        //                 }
        //             }
        //         }
        //     }
        // }
        // panic!("get_owner_address not yet implemented")
    }

    fn get_caller(&self) -> Address {
        self.tx_input.from.clone()
    }

    fn get_balance(&self, _address: &Address) -> RustBigUint {
        // let state = self.mut_context_cell.borrow();
        // match &state.current_tx {
        //     None => panic!("Tx not initialized!"),
        //     Some(tx) => {
        //         match state.accounts.get(&tx.to) {
        //             None => panic!("Account not found!"),
        //             Some(acct) => {
        //                 acct.balance.clone().into()
        //             }
        //         }
        //     }
        // }
        panic!("get balance not yet implemented")
    }

    fn storage_store(&self, key: &[u8], value: &[u8]) {
        let mut mut_context = self.mut_context_cell.borrow_mut();
        mut_context.contract_storage.insert(key.to_vec(), value.to_vec());
    }

    fn storage_load(&self, key: &[u8]) -> Vec<u8> {
        let mut_context = self.mut_context_cell.borrow();
        match mut_context.contract_storage.get(&key.to_vec()) {
            None => Vec::with_capacity(0),
            Some(value) => {
                value.clone()
            },
        }
    }

    #[inline]
    fn storage_load_len(&self, key: &[u8]) -> usize {
        self.storage_load(key).len()
    }

    fn storage_store_bytes32(&self, key: &[u8], value: &[u8; 32]) {
        let mut vector = Vec::with_capacity(32);
        for i in value.iter() {
            vector.push(*i);
        }
        self.storage_store(key, &vector);
    }
    
    fn storage_load_bytes32(&self, key: &[u8]) -> [u8; 32] {
        let value = self.storage_load(key);
        let mut res = [0u8; 32];
        let offset = 32 - value.len();
        if !value.is_empty() {
            res[offset..(value.len()-1 + offset)].clone_from_slice(&value[..value.len()-1]);
            // for i in 0..value.len()-1 {
            //     res[offset+i] = value[i];
            // }
        }
        res
    }

    fn storage_store_big_uint(&self, key: &[u8], value: &RustBigUint) {
        self.storage_store(key, &value.to_bytes_be());
    }

    fn storage_load_big_uint(&self, key: &[u8]) -> RustBigUint {
        let value = self.storage_load(key);
        let bi = BigInt::from_bytes_be(num_bigint::Sign::Plus, value.as_slice());
        bi.into()
    }

    fn storage_store_big_int(&self, key: &[u8], value: &RustBigInt) {
        self.storage_store(key, &value.to_signed_bytes_be());
    }

    fn storage_load_big_int(&self, key: &[u8]) -> RustBigInt {
        let value = self.storage_load(key);
        let bi = BigInt::from_signed_bytes_be(value.as_slice());
        bi.into()
    }

    fn storage_store_i64(&self, key: &[u8], value: i64) {
        self.storage_store_big_int(key, &RustBigInt::from(value));
    }

    fn storage_load_i64(&self, key: &[u8]) -> Option<i64> {
        let bi = self.storage_load_big_int(key);
        bi.value().to_i64()
    }

    #[inline]
    fn get_call_value_big_uint(&self) -> RustBigUint {
        self.tx_input.call_value.clone().into()
    }

    fn send_tx(&self, _to: &Address, _amount: &RustBigUint, _message: &str) {
        // let owner = self.get_sc_address();
        // let mut state = self.mut_context_cell.borrow_mut();
        // match state.accounts.get_mut(&owner) {
        //     None => panic!("Account not found!"),
        //     Some(acct) => {
        //         acct.balance -= amount.value();
        //     }
        // }
        // match state.accounts.get_mut(to) {
        //     None => panic!("Account not found!"),
        //     Some(acct) => {
        //         acct.balance += amount.value();
        //     }
        // }
        panic!("send_tx not yet implemented");
    }

    fn async_call(&self, _to: &Address, _amount: &RustBigUint, _data: &[u8]) {
        panic!("async_call not yet implemented");
    }

    fn get_tx_hash(&self) -> H256 {
        panic!("get_tx_hash not yet implemented");
    }

    fn get_gas_left(&self) -> i64 {
        1000000000
    }

    fn get_block_timestamp(&self) -> u64 {
        0
    }

    fn get_block_nonce(&self) -> u64 {
        0
    }

    fn get_block_round(&self) -> u64 {
        0
    }

    fn get_block_epoch(&self) -> u64 {
        0
    }

    fn sha256(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.input(data);
        hasher.result().into()
    }

    fn keccak256(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.input(data);
        hasher.result().into()
    }
}

impl elrond_wasm::ContractIOApi<RustBigInt, RustBigUint> for TxContext {

    fn get_num_arguments(&self) -> i32 {
        self.tx_input.args.len() as i32
    }

    fn check_not_payable(&self) {
        if self.get_call_value_big_uint() > 0 {
            self.signal_error(err_msg::NON_PAYABLE);
        }
    }

    fn get_argument_len(&self, arg_index: i32) -> usize {
        let arg = self.get_argument_vec(arg_index);
        arg.len()
    }

    fn copy_argument_to_slice(&self, _arg_index: i32, _slice: &mut [u8]) {
        panic!("copy_argument_to_slice not yet implemented")
    }

    fn get_argument_vec(&self, arg_index: i32) -> Vec<u8> {
        let arg_idx_usize = arg_index as usize;
        if arg_idx_usize >= self.tx_input.args.len() {
            panic!("Tx arg index out of range");
        }
        self.tx_input.args[arg_idx_usize].clone()
    }

    fn get_argument_bytes32(&self, arg_index: i32) -> [u8; 32] {
        let arg = self.get_argument_vec(arg_index);
        let mut res = [0u8; 32];
        let offset = 32 - arg.len();
        res[offset..(arg.len()-1 + offset)].clone_from_slice(&arg[..arg.len()-1]);
        res
    }
    
    fn get_argument_big_int(&self, arg_index: i32) -> RustBigInt {
        let bytes = self.get_argument_vec(arg_index);
        BigInt::from_signed_bytes_be(&bytes).into()
    }

    #[inline]
    fn get_argument_big_uint(&self, _arg_index: i32) -> RustBigUint {
        panic!("get_argument_big_uint not yet implemented")
    }

    #[inline]
    fn get_argument_i64(&self, arg_index: i32) -> i64 {
        let bytes = self.get_argument_vec(arg_index);
        let bi = BigInt::from_signed_bytes_be(&bytes);
        if let Some(v) = bi.to_i64() {
            v
        } else {
            panic!("Argument does not fit in an i64.")
        }
    }

    fn finish_slice_u8(&self, slice: &[u8]) {
        let mut v = vec![0u8; slice.len()];
        v.copy_from_slice(slice);
        let mut mut_context = self.mut_context_cell.borrow_mut();
        mut_context.result.result_values.push(v)
    }

    fn finish_bytes32(&self, bytes: &[u8; 32]) {
        self.finish_slice_u8(&*bytes);
    }

    fn finish_big_int(&self, bi: &RustBigInt) {
        self.finish_slice_u8(bi.to_signed_bytes_be().as_slice());
    }

    #[inline]
    fn finish_big_uint(&self, bu: &RustBigUint) {
        self.finish_slice_u8(bu.to_bytes_be().as_slice());
    }
    
    #[inline]
    fn finish_i64(&self, value: i64) {
        self.finish_big_int(&value.into());
    }

    fn signal_error(&self, message: &[u8]) -> ! {
        let s = std::str::from_utf8(message);
        panic!("{}", s.unwrap())
    }

    fn write_log(&self, _topics: &[[u8;32]], _data: &[u8]) {
        println!("write_log not yet implemented");
    }
}
