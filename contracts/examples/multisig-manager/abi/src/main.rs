use elrond_wasm_debug::*;
use multisig_manager::*;

fn main() {
	let contract = MultisigManagerImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
