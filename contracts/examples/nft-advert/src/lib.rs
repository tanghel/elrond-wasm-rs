#![no_std]

mod nft_advert;

use nft_advert::Advert;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ONE_EGLD: u64 = 1000000000000000000u64;

/// Multi-signature manager implementation.
/// Associated names to multisig smart contracts, as well as multisg address lists for wallet addresses.
#[elrond_wasm_derive::contract(NftAdvertImpl)]
pub trait NftAdvert {
	#[storage_mapper("adverts")]
	fn get_adverts(&self) -> MapMapper<Self::Storage, u32, Advert>;

	#[view(getPrice)]
	fn get_price(&self, index: u8) -> BigUint {
		if index >= 245 {
			return BigUint::zero();
		}

		let multiplier = if index < 5 { 16 } else { if index < 65 { 4 } else { 1 } };
		let box_index = index % 5;

		let number_of_pixels = multiplier * if box_index < 3 { 2500 } else { 1250 };

		return BigUint::from(ONE_EGLD).mul(BigUint::from(number_of_pixels as u32)).mul(BigUint::from(8u32)).div(BigUint::from(1000u32));
	}
}
