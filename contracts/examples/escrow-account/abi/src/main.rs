use elrond_wasm_debug::*;
use escrow_account::*;

fn main() {
	let contract = EscrowAccountImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
