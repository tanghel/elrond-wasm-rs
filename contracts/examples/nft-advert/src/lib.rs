#![no_std]

mod nft_advert;

use nft_advert::Advert;

elrond_wasm::imports!();

/// Multi-signature manager implementation.
/// Associated names to multisig smart contracts, as well as multisg address lists for wallet addresses.
#[elrond_wasm_derive::contract(NftAdvertImpl)]
pub trait NftAdvert {
	#[storage_mapper("adverts")]
	fn get_adverts(&self) -> MapMapper<Self::Storage, u32, Advert>;

	#[view(getPrice)]
	fn get_price(&self, index: u8) -> u32 {
		if index >= 245 {
			return 0u32;
		}

		let multiplier = if index < 5 { 16.0 } else { if index < 65 { 4.0 } else { 1.0 } };
		let box_index = index % 5;

		let number_of_pixels = multiplier * if box_index < 3 { 2500.0 } else { 1250.0 };
		let price_per_pixel = 0.008;

		return (number_of_pixels * price_per_pixel) as u32;
	}
}
