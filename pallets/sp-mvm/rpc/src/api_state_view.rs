use codec::{self, Codec};
use std::{sync::Arc};
// use jsonrpc_core::{Error as JsonRpseeError, ErrorCode, Result};
// use jsonrpc_derive::rpc;
use jsonrpsee::{
	core::{Error as JsonRpseeError},
	types::error::{CallError, ErrorObject},
};

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_mvm_rpc_runtime::{ MVMApiRuntime};
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
pub use crate::move_types::MoveModuleBytecode;
use move_core_types::{
	account_address::AccountAddress,
	language_storage::{ModuleId, StructTag},
	resolver::{ModuleResolver, ResourceResolver},
};
use move_binary_format::{layout::GetModule, CompiledModule};
use anyhow::anyhow;
pub struct ApiStateView<C, BlockHash, AccountId, Block> {
	client: Arc<C>,
	account_id: AccountId,
	at: Option<BlockHash>,
	_marker: std::marker::PhantomData<Block>,
}
impl<C, Block, AccountId> ApiStateView<C, <Block as BlockT>::Hash, AccountId, Block>
where
	Block: BlockT,
{
	pub fn new(client: Arc<C>, account_id: AccountId, at: Option<<Block as BlockT>::Hash>) -> Self {
		Self { client, account_id, at, _marker: Default::default() }
	}
}
impl<C, Block, AccountId> ModuleResolver
	for ApiStateView<C, <Block as BlockT>::Hash, AccountId, Block>
where
	Block: BlockT,
	AccountId: Clone + std::fmt::Display + Codec,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: MVMApiRuntime<Block, AccountId>,
{
	type Error = anyhow::Error;

	fn get_module(&self, module_id: &ModuleId) -> anyhow::Result<Option<Vec<u8>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));
		let bytes: Option<Vec<u8>> = api
			.get_module(&at, bcs_alt::to_bytes(module_id).unwrap())
			.map_err(runtime_error_into_rpc_err4)?
			.map_err(runtime_error_into_rpc_err5)?;
		Ok(bytes)
	}
}
impl<C, Block, AccountId> ResourceResolver
	for ApiStateView<C, <Block as BlockT>::Hash, AccountId, Block>
where
	Block: BlockT,
	AccountId: Clone + std::fmt::Display + Codec,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: MVMApiRuntime<Block, AccountId>,
{
	type Error = anyhow::Error;

	fn get_resource(
		&self,
		_address: &AccountAddress,
		tag: &StructTag,
	) -> anyhow::Result<Option<Vec<u8>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));
		let bytes: Option<Vec<u8>> = api
			.get_resource(&at, self.account_id.clone(), bcs_alt::to_bytes(tag).unwrap())
			.map_err(runtime_error_into_rpc_err4)?
			.map_err(runtime_error_into_rpc_err5)?;
		Ok(bytes)
	}
}


impl<C, Block, AccountId> GetModule
	for ApiStateView<C, <Block as BlockT>::Hash, AccountId, Block>
where
	Block: BlockT,
	AccountId: Clone + std::fmt::Display + Codec,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: MVMApiRuntime<Block, AccountId>,
{
	type Error = anyhow::Error;

    fn get_module_by_id(&self, id: &ModuleId) -> Result<Option<CompiledModule>, Self::Error> {
		if let Some(bytes) = self.get_module(id)? {
			let module = CompiledModule::deserialize(&bytes)
				.map_err(|e| anyhow!("Failure deserializing module {:?}: {:?}", id, e))?;
			Ok(Some(module))
		} else {
			Ok(None)
		}
	}
}

const RUNTIME_ERROR: i32 = 500;
fn runtime_error_into_rpc_err4(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(RUNTIME_ERROR, "ABI error", Some(format!("{:?}", err))))
		.into()
}

fn runtime_error_into_rpc_err5(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Error from method",
		Some(format!("{:?}", err)),
	))
	.into()
}
