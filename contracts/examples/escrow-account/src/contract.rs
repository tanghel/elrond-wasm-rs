#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Contract<BigUint: BigUintApi> {
	pub buyer: Address,
	pub seller: Address,
	pub refund_period: u64,
	pub milestones: Vec<Milestone<BigUint>>
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct Milestone<BigUint: BigUintApi> {
	pub amount: BigUint,
	pub date: u64
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct ContractResult<BigUint: BigUintApi> {
	pub buyer: Address,
	pub seller: Address,
	pub status: ContractStatus,
	pub milestones: Vec<MilestoneResult<BigUint>>
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct MilestoneResult<BigUint: BigUintApi> {
	pub amount: BigUint,
	pub date: u64,
	pub status: MilestoneStatus,
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
	Ongoing,
	Cancelled,
	Refunded,
	Fulfilled
}