//! Substrate Node Template CLI library.
use move_vm::genesis::GenesisConfig;

const MODULE_NAME: &[u8] = "Genesis".as_bytes();
const FUNC_NAME: &[u8] = "initialize".as_bytes();
/// Module Name
pub type ModuleName = Vec<u8>;
/// Function Name
pub type FunctionName = Vec<u8>;
/// Function Arguments
pub type FunctionArgs = Vec<Vec<u8>>;

/// Build configuration to call initialize functions on standard library.
pub fn build() -> (ModuleName, FunctionName, FunctionArgs) {
    // We use standard arguments.
    let genesis: GenesisConfig = Default::default();

    (
        MODULE_NAME.to_vec(),
        FUNC_NAME.to_vec(),
        genesis.init_func_config.unwrap().args,
    )
}
