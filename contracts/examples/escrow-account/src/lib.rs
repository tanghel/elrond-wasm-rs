#![no_std]

elrond_wasm::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
/// See the readme file for more detailed documentation.
#[elrond_wasm_derive::contract(EscrowAccountImpl)]
pub trait EscrowAccount {
	
}
