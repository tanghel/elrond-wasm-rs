use elrond_wasm_debug::*;
use multisig_deployer::*;

fn main() {
	let contract = MultisigDeployerImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
