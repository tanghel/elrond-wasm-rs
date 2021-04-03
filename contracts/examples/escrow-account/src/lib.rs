#![no_std]

elrond_wasm::imports!();

mod contract;
use contract::{Contract, Milestone, ContractStatus, MilestoneStatus};

#[elrond_wasm_derive::contract(EscrowAccountImpl)]
pub trait EscrowAccount {
	#[storage_mapper("contracts")]
	fn get_contracts(&self) -> VecMapper<Self::Storage, Contract<BigUint>>;

	#[view(getContractStatus)]
	#[storage_get("contract_status")]
	fn get_contract_status(&self, contract_id: usize) -> ContractStatus;

	#[storage_set("contract_status")]
	fn set_contract_status(&self, contract_id: usize, status: ContractStatus);

	#[view(getContractCancelledTimestamp)]
	#[storage_get("contract_cancelled_timestamp")]
	fn get_contract_cancelled_timestamp(&self, contract_id: usize) -> u64;

	#[storage_set("contract_cancelled_timestamp")]
	fn set_contract_cancelled_timestamp(&self, contract_id: usize, timestamp: u64);

	#[view(getMilestoneStatus)]
	#[storage_get("milestone_status")]
	fn get_milestone_status(&self, contract_id: usize) -> MilestoneStatus;

	#[storage_set("milestone_status")]
	fn set_milestone_status(&self, contract_id: usize, status: MilestoneStatus);

	#[view(getMilestoneIndex)]
	#[storage_get("milestone_index")]
	fn get_milestone_index(&self, contract_id: usize) -> usize;

	#[storage_set("milestone_index")]
	fn set_milestone_index(&self, contract_id: usize, index: usize);

	#[storage_mapper("contracts_by_address")]
	fn contracts_by_address(&self, owner: &Address) -> VecMapper<Self::Storage, usize>;

	#[endpoint(propose)]
	fn propose(&self, contract: Contract<BigUint>) -> SCResult<usize> {
		let contract_id = self.get_contracts().push(&contract);

		self.set_contract_status(contract_id, ContractStatus::Proposed);
		self.set_milestone_status(contract_id, MilestoneStatus::Unpaid);
		self.set_milestone_index(contract_id, 1usize);

		Ok(contract_id)
	}

	#[endpoint(start)]
	#[payable("EGLD")]
	fn start(&self, contract_id: usize, #[payment] payment: BigUint) -> SCResult<()> {
		let contract = self.get_contracts().get(contract_id);

		require!(contract.buyer == self.get_caller(), "Only the buyer can start contract!");

		let contract_status = self.get_contract_status(contract_id);
		require!(contract_status == ContractStatus::Proposed, "Contract status must be proposed!");

		let mut contract_amount = BigUint::from(0u32);
		for milestone in contract.milestones {
			contract_amount += milestone.amount
		}

		require!(contract_amount == payment, "Invalid payment amount");

		self.set_contract_status(contract_id, ContractStatus::Ongoing);

		Ok(())
	}

	#[endpoint(cancel)]
	fn cancel(&self, contract_id: usize) -> SCResult<()> {
		let contract = self.get_contracts().get(contract_id);

		require!(contract.buyer == self.get_caller(), "Only the buyer can cancel contract!");

		let contract_status = self.get_contract_status(contract_id);
		require!(contract_status == ContractStatus::Ongoing, "Contract status must be ongoing!");

		self.set_contract_status(contract_id, ContractStatus::Cancelled);
		self.set_contract_cancelled_timestamp(contract_id, self.get_block_timestamp());

		Ok(())
	}

	#[endpoint(refund)]
	fn refund(&self, contract_id: usize) -> SCResult<()> {
		let contract = self.get_contracts().get(contract_id);

		require!(contract.buyer == self.get_caller(), "Only the buyer can refund contract!");

		let contract_status = self.get_contract_status(contract_id);
		require!(contract_status == ContractStatus::Cancelled, "Contract status must be cancelled!");

		let contract_cancelled_timestamp = self.get_contract_cancelled_timestamp(contract_id);
		let refund_period = contract.refund_period;
		let now = self.get_block_timestamp();
		require!(now >= contract_cancelled_timestamp + refund_period, "Refund period not yet over");

		let amount_to_refund = self.get_contract_remaining_amount(contract_id);

		let data = BoxedBytes::from(&b"Contract refund"[..]);

		self.send().direct_egld(&contract.buyer, &amount_to_refund, data.as_slice());

		self.set_contract_status(contract_id, ContractStatus::Refunded);

		Ok(())
	}

	#[endpoint(paymentRelease)]
	fn payment_release(&self, contract_id: usize) -> SCResult<()> {
		let contract = self.get_contracts().get(contract_id);

		require!(contract.buyer == self.get_caller(), "Only the buyer can release payment!");

		let contract_status = self.get_contract_status(contract_id);
		require!(contract_status == ContractStatus::Ongoing, "Contract status must be ongoing!");

		let milestone_status = self.get_milestone_status(contract_id);
		require!(milestone_status != MilestoneStatus::Released, "Milestone already released!");

		self.set_milestone_status(contract_id, MilestoneStatus::Released);

		Ok(())
	}

	#[endpoint(paymentBlock)]
	fn payment_block(&self, contract_id: usize) -> SCResult<()> {
		let contract = self.get_contracts().get(contract_id);

		require!(contract.buyer == self.get_caller(), "Only the buyer can block payment!");

		let contract_status = self.get_contract_status(contract_id);
		require!(contract_status == ContractStatus::Ongoing, "Contract status must be ongoing!");

		let milestone_status = self.get_milestone_status(contract_id);
		require!(milestone_status != MilestoneStatus::Blocked, "Milestone already blocked!");

		self.set_milestone_status(contract_id, MilestoneStatus::Blocked);

		Ok(())
	}

	#[endpoint(paymentReceive)]
	fn payment_receive(&self, contract_id: usize) -> SCResult<()> {
		let contract = self.get_contracts().get(contract_id);

		require!(contract.seller == self.get_caller(), "Only the seller can receive payment!");

		let contract_status = self.get_contract_status(contract_id);
		require!(contract_status == ContractStatus::Ongoing, "Contract status must be ongoing!");

		if let Some(milestone) = self.get_milestone(contract_id) {
			let milestone_status = self.get_milestone_status(contract_id);
			let is_past_date = milestone.date > 0 && self.get_block_timestamp() >= milestone.date;
			let is_released = milestone_status == MilestoneStatus::Released;
	
			require!(is_released || is_past_date, "Milestone is not released yet and/or target date not reached!");
	
			self.send().direct_egld(&contract.seller, &milestone.amount, b"Contract payment");

			let new_milestone_index = self.get_milestone_index(contract_id) + 1;
			self.set_milestone_index(contract_id, new_milestone_index);
			self.set_milestone_status(contract_id, MilestoneStatus::Unpaid);

			if new_milestone_index > contract.milestones.len() {
				self.set_contract_status(contract_id, ContractStatus::Fulfilled);
			}
		}

		Ok(())
	}

	#[view(getMilestoneAmount)]
	fn get_milestone_amount(&self, contract_id: usize) -> BigUint {
		if let Some(milestone) = self.get_milestone(contract_id) {
			return milestone.amount;
		}

		return BigUint::from(0u32);
	}

	#[view(getContractTotalAmount)]
	fn get_contract_total_amount(&self, contract_id: usize) -> BigUint {
		let contract = self.get_contracts().get(contract_id);

		let mut total_amount = BigUint::from(0u32);
		for milestone in contract.milestones {
			total_amount += milestone.amount;
		}

		return total_amount;
	}

	#[view(getContractRemainingAmount)]
	fn get_contract_remaining_amount(&self, contract_id: usize) -> BigUint {
		let contract = self.get_contracts().get(contract_id);

		let milestone_index = self.get_milestone_index(contract_id);

		let mut total_amount = BigUint::from(0u32);
		let mut index = 1;
		for milestone in contract.milestones {
			if index >= milestone_index {
				total_amount += milestone.amount;
			}

			index = index + 1;
		}

		return total_amount;
	}

	#[view(getRefundPeriodRemainingSeconds)]
	fn get_refund_period_remaining_seconds(&self, contract_id: usize) -> u64 {
		let contract = self.get_contracts().get(contract_id);

		let contract_cancelled_timestamp = self.get_contract_cancelled_timestamp(contract_id);
		let refund_period = contract.refund_period;
		let now = self.get_block_timestamp();

		if now >= contract_cancelled_timestamp + refund_period {
			return 0;
		}

		return contract_cancelled_timestamp + refund_period - now;
	}

	#[view(getMilestoneReleaseRemainingSeconds)]
	fn get_milestone_release_remaining_seconds(&self, contract_id: usize) -> u64 {
		if let Some(milestone) = self.get_milestone(contract_id) {
			if milestone.date == 0 {
				return 0;
			}
			
			let milestone_status = self.get_milestone_status(contract_id);
			if milestone_status != MilestoneStatus::Unpaid {
				return 0;
			}

			let now = self.get_block_timestamp();

			if milestone.date > now {
				return milestone.date - now;
			}
		}

		return 0;
	}

	fn get_milestone(&self, contract_id: usize) -> Option<Milestone<BigUint>> {
		let contract = self.get_contracts().get(contract_id);
		let milestone_index = self.get_milestone_index(contract_id);

		let mut current_index = 1;
		for milestone in contract.milestones {
			if current_index == milestone_index {
				return Some(milestone);
			}
			
			current_index = current_index + 1;
		}

		return None;
	}
}
