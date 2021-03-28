#![no_std]

elrond_wasm::imports!();

mod transaction;

use transaction::Transaction;

#[elrond_wasm_derive::contract(EscrowAccountImpl)]
pub trait EscrowAccount {
	#[storage_mapper("transactions")]
	fn transactions_mapper(&self) -> VecMapper<Self::Storage, Transaction<BigUint>>;

	#[endpoint(createTransaction)]
	#[payable("EGLD")]
	fn create_transaction(&self, #[payment] amount: BigUint, seller: Address) -> SCResult<usize> {
		let buyer = self.get_caller();

		let transaction = Transaction
		{
			buyer,
			seller,
			amount
		};

		let transaction_id = self.transactions_mapper().push(&transaction);

		Ok(transaction_id)
	}

	#[endpoint(payoutTransaction)]
	fn payout_transaction(&self, transaction_id: usize) -> SCResult<()> {
		let buyer = self.get_caller();

		require!(
			!self.transactions_mapper().item_is_empty_unchecked(transaction_id), 
			"Could not identify transaction"
		);

		let transaction = self.transactions_mapper().get(transaction_id);

		require!(transaction.buyer == buyer, "Only the buyer can trigger payout!");

		let to = transaction.seller;
		let amount = transaction.amount;
		let data = BoxedBytes::empty();

		self.transactions_mapper().clear_entry_unchecked(transaction_id);
		self.send().direct_egld(&to, &amount, data.as_slice());

		Ok(())
	}

	#[view(getTransactionBuyer)]
	fn get_transaction_buyer(&self, transaction_id: usize) -> Address {
		let transaction = self.transactions_mapper().get_unchecked(transaction_id);
		Ok(transaction.buyer)
	}

	#[view(getTransactionSeller)]
	fn get_transaction_seller(&self, transaction_id: usize) -> Address {
		let transaction = self.transactions_mapper().get_unchecked(transaction_id);
		transaction.seller
	}

	#[view(getTransactionAmount)]
	fn get_transaction_amount(&self, transaction_id: usize) -> BigUint {
		let transaction = self.transactions_mapper().get_unchecked(transaction_id);
		transaction.amount
	}
}
