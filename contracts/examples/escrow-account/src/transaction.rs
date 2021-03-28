use elrond_wasm::types::{Address};
use elrond_wasm::api::BigUintApi;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Transaction<BigUint: BigUintApi> {
	pub buyer: Address,
	pub seller: Address,
	pub amount: BigUint,
}
