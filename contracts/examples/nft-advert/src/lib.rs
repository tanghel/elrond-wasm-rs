#![no_std]

mod nft_advert;

use nft_advert::Advert;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ONE_EGLD: u64 = 1000000000000000000u64;
const ADVERTS_COUNT: u8 = 245;

#[elrond_wasm_derive::contract(NftAdvertImpl)]
pub trait NftAdvert {
	#[storage_mapper("adverts")]
	fn get_adverts(&self) -> MapMapper<Self::Storage, u8, Advert>;

	#[payable("EGLD")]
	#[endpoint]
	fn assign(&self, index: u8, #[payment] amount: BigUint, title: BoxedBytes, url: BoxedBytes, image: BoxedBytes) -> SCResult<()> {
		require!(index < ADVERTS_COUNT, "Invalid advert index");

		require!(!self.get_adverts().contains_key(&index), "Advert already assigned!");

		require!(amount == self.get_price(index), "Invalid amount");

		let owner = self.get_caller();

		self.get_adverts().insert(index, Advert {
			title,
			url,
			image,
			owner
		});

		Ok(())
	}

	#[endpoint]
	fn update(&self, index: u8, title: BoxedBytes, url: BoxedBytes, image: BoxedBytes) -> SCResult<()> {
		require!(index < ADVERTS_COUNT, "Invalid advert index");

		if let Some(advert) = self.get_adverts().get(&index) {
			let owner = self.get_caller();

			require!(advert.owner == owner, "Caller is not owner of said advert.");

			self.get_adverts().insert(index, Advert {
				title,
				url,
				image,
				owner
			});

			Ok(())
		} else {
			sc_error!("Caller is not owner of said advert.")
		}
	}

	#[endpoint]
	fn clear(&self, index: u8) -> SCResult<()> {
		require!(index < ADVERTS_COUNT, "Invalid advert index");

		if let Some(advert) = self.get_adverts().get(&index) {
			require!(advert.owner == self.get_caller(), "Caller is not owner of said advert.");

			self.get_adverts().remove(&index);

			Ok(())
		} else {
			sc_error!("Caller is not owner of said advert.")
		}
	}

	#[endpoint]
	fn transfer(&self, index: u8, new_owner: Address) -> SCResult<()> {
		require!(index < ADVERTS_COUNT, "Invalid advert index");

		if let Some(advert) = self.get_adverts().get(&index) {
			require!(advert.owner == self.get_caller(), "Caller is not owner of said advert.");

			self.get_adverts().insert(index, Advert {
				title: advert.title,
				url: advert.url,
				image: advert.image,
				owner: new_owner
			});

			Ok(())
		} else {
			sc_error!("Caller is not owner of said advert.")
		}
	}

	#[view(getPrice)]
	fn get_price(&self, index: u8) -> BigUint {
		if index >= ADVERTS_COUNT {
			return BigUint::zero();
		}

		let multiplier = if index < 5 { 16 } else { if index < 65 { 4 } else { 1 } };
		let box_index = index % 5;

		let number_of_pixels = multiplier * if box_index < 3 { 2500 } else { 1250 };

		return BigUint::from(ONE_EGLD).mul(BigUint::from(number_of_pixels as u32)).mul(BigUint::from(8u32)).div(BigUint::from(1000u32));
	}

	#[view(getAllAdverts)]
	fn get_all_adverts(&self) -> MultiResultVec<Advert> {
		self.get_adverts().values().collect()
	}
}
