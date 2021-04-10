use elrond_wasm::types::{Address, BoxedBytes};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Advert {
	pub title: BoxedBytes,
	pub url: BoxedBytes,
	pub image: BoxedBytes,
	pub owner: Address,
}
