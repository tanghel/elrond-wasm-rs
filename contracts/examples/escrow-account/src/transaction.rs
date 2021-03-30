#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Contract {
	pub buyer: Address,
	pub seller: Address,
	pub buyer_copy: BoxedBytes,
	pub seller_copy: BoxedBytes,
	pub status: ContractStatus,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Milestone<BigUint: BigUintApi> {
	pub amount: BigUint,
	pub date: BigUint,
	pub status: MilestoneStatus,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Copy, TypeAbi)]
pub enum Party {
	Buyer,
	Seller
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Copy, TypeAbi)]
pub enum MilestoneStatus {
	Unpaid,
	Released,
	Blocked,
	Paid
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Copy, TypeAbi)]
pub enum ContractStatus {
	Proposed,
	Signed,
	Cancelled,
	Refunded,
	Fulfilled
}