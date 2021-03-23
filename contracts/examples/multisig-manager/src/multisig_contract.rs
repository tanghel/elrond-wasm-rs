use elrond_wasm::types::{Address, BoxedBytes};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct MultisigContractInfo {
	pub address: Address,
	pub name: BoxedBytes,
}
