#![no_std]

elrond_wasm::imports!();

mod transaction;

use transaction::{Contract, Milestone, ContractStatus, MilestoneStatus, Party};

#[elrond_wasm_derive::contract(EscrowAccountImpl)]
pub trait EscrowAccount {
	#[storage_mapper("contracts")]
	fn contracts_mapper(&self) -> VecMapper<Self::Storage, Contract>;

	#[endpoint(propose)]
	fn propose(&self, data: BoxedBytes, party: Party) -> SCResult<usize> {
		let status = ContractStatus::Proposed;

		if party == Party::Buyer {
			let buyer = self.get_caller();
			let seller = Address::zero();
			let buyer_copy = data;
			let seller_copy = BoxedBytes::empty();

			let contract = Contract
			{
				buyer,
				seller,
				buyer_copy,
				seller_copy,
				status
			};

			let contract_id = self.contracts_mapper().push(&contract);
			Ok(contract_id)
		} else if party == Party::Seller {
			let buyer = Address::zero();
			let seller = self.get_caller();
			let buyer_copy = BoxedBytes::empty();
			let seller_copy = data;

			let contract = Contract
			{
				buyer,
				seller,
				buyer_copy,
				seller_copy,
				status
			};

			let contract_id = self.contracts_mapper().push(&contract);
			Ok(contract_id)
		} else {
			Ok(0)
		}
	}

	#[endpoint(sign)]
	#[payable("EGLD")]
	fn sign(&self, contract_id: usize, data: BoxedBytes) -> SCResult<()> {
		Ok(())
	}

	#[endpoint(cancel)]
	fn cancel(&self, contract_id: usize) -> SCResult<()> {
		Ok(())
	}

	#[endpoint(refund)]
	fn refund(&self, contract_id: usize) -> SCResult<()> {
		Ok(())
	}

	#[endpoint(paymentRelease)]
	fn payment_release(&self, contract_id: usize, milestone_id: usize) -> SCResult<()> {
		Ok(())
	}

	#[endpoint(paymentBlock)]
	fn payment_block(&self, contract_id: usize, milestone_id: usize) -> SCResult<()> {
		Ok(())
	}

	#[endpoint(paymentReceive)]
	fn payment_receive(&self, contract_id: usize, milestone_id: usize) -> SCResult<()> {
		Ok(())
	}
}
