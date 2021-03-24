#![no_std]

mod multisig_contract;

use multisig_contract::MultisigContractInfo;

elrond_wasm::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
/// See the readme file for more detailed documentation.
#[elrond_wasm_derive::contract(MultisigManagerImpl)]
pub trait MultisigManager {
	#[storage_mapper("multisigList")]
	fn get_multisig_list(&self, owner: &Address) -> MapMapper<Self::Storage, Address, BoxedBytes>;

	#[storage_mapper("multisigNames")]
	fn get_multisig_names(&self) -> MapMapper<Self::Storage, Address, BoxedBytes>;

	fn register_multisig_user_contract(&self, user_address: &Address, contract_address: Address) {
		self.get_multisig_list(user_address).insert(contract_address, BoxedBytes::empty());
	}

	fn unregister_multisig_user_contract(&self, user_address: &Address, contract_address: &Address) {
		self.get_multisig_list(user_address).remove(contract_address);
	}

	#[endpoint(registerMultisigContract)]
	fn register_multisig_contract(&self, contract_address: Address) -> SCResult<()> {
		let user_address = self.get_caller();

		self.register_multisig_user_contract(&user_address, contract_address);

		Ok(())
	}

	#[endpoint(registerMultisigUser)]
	fn register_multisig_user(&self, user_address: Address) -> SCResult<()> {
		let contract_address = self.get_caller();

		self.register_multisig_user_contract(&user_address, contract_address);

		Ok(())
	}

	#[endpoint(unregisterMultisigContract)]
	fn unregister_multisig_contract(&self, contract_address: Address) -> SCResult<()> {
		let user_address = self.get_caller();

		self.unregister_multisig_user_contract(&user_address, &contract_address);

		Ok(())
	}

	#[endpoint(unregisterMultisigUser)]
	fn unregister_multisig_user(&self, user_address: Address) -> SCResult<()> {
		let contract_address = self.get_caller();

		self.unregister_multisig_user_contract(&user_address, &contract_address);

		Ok(())
	}

	#[view(getMultisigContractAddresses)]
	fn get_multisig_contract_addresses(&self, user_address: Address) -> Vec<Address> {
		self.get_multisig_list(&user_address).keys().collect()
	}

	#[view(getMultisigContracts)]
	fn get_multisig_contracts(&self, user_address: Address) -> MultiResultVec<MultisigContractInfo> {
		let addresses = self.get_multisig_contract_addresses(user_address);

		let mut result = Vec::new();
		for address in addresses {
			let name = self.get_multisig_contract_name(&address);

			result.push(MultisigContractInfo {
				address,
				name
			})
		}

		result.into()
	}

	#[view(getMultisigContractName)]
	fn get_multisig_contract_name(&self, multisig_address: &Address) -> BoxedBytes {
		self.get_multisig_names()
			.get(&multisig_address)
			.unwrap_or_else(|| BoxedBytes::empty())
	}
}
