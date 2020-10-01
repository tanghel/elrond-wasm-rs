
extern crate use_module;
use use_module::*;
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap {
    let mut contract_map = ContractMap::new();
    contract_map.register_contract(
        "file:../output/use_module.wasm",
        Box::new(|mock_ref| Box::new(UseModuleImpl::new(mock_ref))));
    contract_map
}

#[test]
fn use_module_features() {
    let contract_map = contract_map();

    let mut state = BlockchainMock::new();

    parse_execute_mandos("mandos/use_module_features.scen.json", &mut state, &contract_map);
}
