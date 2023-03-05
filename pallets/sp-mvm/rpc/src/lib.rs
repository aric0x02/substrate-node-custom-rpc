use std::sync::Arc;
use std::convert::From;
use codec::{self, Codec};
// use jsonrpc_core::{Error as JsonRpseeError, ErrorCode, Result};
// use jsonrpc_derive::rpc;
use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult as Result},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};

use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT},
};
use sp_api::ProvideRuntimeApi;
use sp_mvm_rpc_runtime::{MVMApiRuntime, types::MVMApiEstimation};
// use frame_support::weights::Weight;
use serde::{Serialize, Deserialize};
use fc_rpc_core::types::Bytes;
pub mod bytecode;
pub mod addr;
pub mod address;
pub mod constant;
pub mod info;
pub mod fn_call;
pub mod model;
pub mod wrappers;
pub mod move_types;
pub mod convert;
pub use crate::move_types::MoveModuleBytecode;
use anyhow::{bail, ensure, format_err, Context as AnyhowContext, Result as AnyHowResult};
use move_core_types::{
account_address::AccountAddress,
resolver::{ModuleResolver, ResourceResolver},
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout,MoveFieldLayout},
};

pub struct ApiStateView<C,BlockHash,AccountId,Block> {
    client: Arc<C>,
    account_id: AccountId,
    at: Option<BlockHash>,
    _marker: std::marker::PhantomData<Block>,
}
impl<C,Block,AccountId> ApiStateView<C,<Block as BlockT>::Hash,AccountId,Block>
where
    Block: BlockT,  {
    pub fn new(client: Arc<C>,    account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>) -> Self {
        Self { client,account_id,at, _marker: Default::default(),}
    }
}
impl<C, Block,AccountId>  ModuleResolver for ApiStateView<C, <Block as BlockT>::Hash,AccountId,Block> 
where
    Block: BlockT,
    AccountId: Clone + std::fmt::Display + Codec,
    C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: MVMApiRuntime<Block, AccountId>, {


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
impl<C, Block,AccountId>  ResourceResolver for ApiStateView<C, <Block as BlockT>::Hash,AccountId,Block> 
where
    Block: BlockT,
    AccountId: Clone + std::fmt::Display + Codec,
    C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: MVMApiRuntime<Block, AccountId>,
 {

    type Error = anyhow::Error;

    fn get_resource(&self, address: &AccountAddress, tag: &StructTag) -> anyhow::Result<Option<Vec<u8>>> {
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

// Estimation struct with serde.
#[derive(Serialize, Deserialize)]
pub struct Estimation {
    pub gas_used: u64,
    pub status_code: u64,
}

impl From<MVMApiEstimation> for Estimation {
    fn from(e: MVMApiEstimation) -> Self {
        Self {
            gas_used: e.gas_used,
            status_code: e.status_code,
        }
    }
}

// RPC calls.
#[rpc(client, server)]
pub trait MVMApiRpc<BlockHash, AccountId> {
    #[method(name = "mvm_gasToWeight")]
    fn gas_to_weight(&self, gas: u64, at: Option<BlockHash>) -> Result<u64>;

    #[method(name = "mvm_weightToGas")]
    fn weight_to_gas(&self, weight: u64, at: Option<BlockHash>) -> Result<u64>;

    #[method(name = "mvm_estimateGasPublish")]
    fn estimate_gas_publish(
        &self,
        account: AccountId,
        module_bc: Bytes,
        gas_limit: u64,
        at: Option<BlockHash>,
    ) -> Result<Estimation>;

    #[method(name = "mvm_estimateGasExecute")]
    fn estimate_gas_execute(
        &self,
        account: AccountId,
        tx_bc: Bytes,
        gas_limit: u64,
        at: Option<BlockHash>,
    ) -> Result<Estimation>;

    #[method(name = "mvm_getResource")]
    fn get_resource(
        &self,
        account_id: AccountId,
        tag: Bytes,
        at: Option<BlockHash>,
    ) -> Result<Option<Bytes>>;

    #[method(name = "mvm_getModuleABI")]
    fn get_module_abi(&self, module_id: Bytes, at: Option<BlockHash>) -> Result<Option<Bytes>>;

    #[method(name = "mvm_getModule")]
    fn get_module(&self, module_id: Bytes, at: Option<BlockHash>) -> Result<Option<Bytes>>;

    #[method(name = "mvm_encodeSubmission")]
    fn encode_submission(&self, function: Vec<Bytes>, arguments: Vec<Bytes>, type_parameters: Vec<Bytes>,at: Option<BlockHash>) -> Result<Option<Bytes>>;

    #[method(name = "mvm_getModuleABIs")]
    fn get_module_abis(&self, module_id: Bytes, at: Option<BlockHash>) -> Result<Option<Bytes>>;
    
    #[method(name = "mvm_getModuleABIs2")]
    fn get_module_abis2(&self, module_id: Bytes, at: Option<BlockHash>) -> Result<Option<MoveModuleBytecode>>;

    #[method(name = "mvm_getResources")]
    fn get_resources(
        &self,
        account_id: AccountId,
        tag: Bytes,
        at: Option<BlockHash>,
    ) -> Result<Option<Bytes>>;
    #[method(name = "mvm_getResources2")]
    fn get_resources2(
        &self,
        account_id: AccountId,
        tag: Bytes,
        at: Option<BlockHash>,
    ) -> Result<Option<Bytes>>;
}

pub struct MVMApi<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> MVMApi<C, P> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> MVMApiRpcServer<<Block as BlockT>::Hash, AccountId> for MVMApi<C, Block>
where
    Block: BlockT,
    AccountId: Clone + std::fmt::Display + Codec,
    C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: MVMApiRuntime<Block, AccountId>,
{
    fn gas_to_weight(&self, gas: u64, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let res = api.gas_to_weight(&at, gas);

        res.map_err(runtime_error_into_rpc_err)
    }

    fn weight_to_gas(&self, weight: u64, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let res = api.weight_to_gas(&at, weight);

        res.map_err(runtime_error_into_rpc_err)
    }

    fn estimate_gas_publish(
        &self,
        account: AccountId,
        module_bc: Bytes,
        gas_limit: u64,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Estimation> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let res = api
            .estimate_gas_publish(&at, account, module_bc.into_vec(), gas_limit)
            .map_err(runtime_error_into_rpc_err)?;

        let mvm_estimation = res.map_err(runtime_error_into_rpc_err2)?;

        Ok(Estimation::from(mvm_estimation))
    }

    fn estimate_gas_execute(
        &self,
        account: AccountId,
        tx_bc: Bytes,
        gas_limit: u64,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Estimation> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let res = api
            .estimate_gas_execute(&at, account, tx_bc.into_vec(), gas_limit)
            .map_err(runtime_error_into_rpc_err)?;

        let mvm_estimation = res.map_err(runtime_error_into_rpc_err3)?;

        Ok(Estimation::from(mvm_estimation))
    }

    fn get_resource(
        &self,
        account_id: AccountId,
        tag: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<Bytes>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let f: Option<Vec<u8>> = api
            .get_resource(&at, account_id, tag.into_vec())
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err5)?;
        Ok(f.map(Into::into))
    }

    fn get_module_abi(
        &self,
        module_id: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<Bytes>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let f: Option<Vec<u8>> = api
            .get_module_abi(&at, module_id.into_vec())
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err5)?;
        Ok(f.map(Into::into))
    }

    fn get_module(
        &self,
        module_id: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<Bytes>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let f: Option<Vec<u8>> = api
            .get_module(&at, module_id.into_vec())
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err5)?;
        Ok(f.map(Into::into))
    }

    fn encode_submission(
        &self,
        function: Vec<Bytes>,  
        arguments: Vec<Bytes>, 
        type_parameters: Vec<Bytes>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<Bytes>> {
       
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));
        let ff = function.into_iter().map(|func|String::from_utf8(func.into_vec()).unwrap()).collect::<Vec<String>>();
        let ((module_id,module_address),module_name,func) = (crate::fn_call::parse_function_string(&ff[0],&ff[1]).unwrap(),ff[1].clone(),ff[2].clone());
 println!("{:?},{:?},{:?},{:?},{:?}",module_id,module_address,ff[0],module_name,func);
        let f: Option<Vec<u8>> = api
            .get_module(&at, module_id.unwrap())
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err4)?;
println!("make_function_call====");
        let f = crate::fn_call::make_function_call(
&f.as_ref().unwrap(),module_address,module_name,func,type_parameters.into_iter().
map(|a| String::from_utf8(a.into_vec()).unwrap()).
collect(),arguments.into_iter().map(|a| String::from_utf8(a.into_vec()).unwrap()).collect())
.map_err(runtime_error_into_rpc_err4).ok();
println!("make_function_call=result==={:?}===",f);
//   MoveModuleBytecode::new(module.clone())
//                             .try_parse_abi()
//                             .context("Failed to parse move module ABI")
//                             .map_err(|err| {
//                                 BasicErrorWith404::internal_with_code(
//                                     err,
//                                     AptosErrorCode::InternalError,
//                                     &self.latest_ledger_info,
//                                 )
//                             })?,

        Ok(f.map(Into::into))
    }

    fn get_module_abis(
        &self,
        module_id: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<Bytes>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let f: Option<Vec<u8>> = api
            .get_module(&at, module_id.into_vec())
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err4)?;
 let f = crate::fn_call::make_abi(&f.as_ref().unwrap()).map_err(runtime_error_into_rpc_err4).ok();
        let ff=serde_json::to_vec(&f.as_ref().unwrap()).ok();
        println!("test_get_module_abis=result==={:?}=={:?}=",f,ff);
        // let f:Option<Vec<u8>>=Some(ff.bytes().collect());
        let f=ff;
        Ok(f.map(Into::into))
    }

 fn get_module_abis2(
        &self,
        module_id: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<MoveModuleBytecode>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let f: Option<Vec<u8>> = api
            .get_module(&at, module_id.into_vec())
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err4)?;
        let f = crate::fn_call::make_abi(&f.as_ref().unwrap()).map_err(runtime_error_into_rpc_err4).ok();
        // let ff=serde_json::to_vec(&f.as_ref().unwrap()).ok();
        println!("test_get_module_abis2=result==={:?}===",f);
        // let f:Option<Vec<u8>>=Some(ff.bytes().collect());
        Ok(f.map(Into::into))
    }

    fn get_resources(
        &self,
        account_id: AccountId,
        tag: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<Bytes>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));
        let (tag_bcs,tag,module_id)=convert::parse_struct_tag_string(tag.into_vec()).unwrap();
        let f: Option<Vec<u8>> = api
            .get_resource(&at, account_id, tag_bcs)
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err5)?;
     let module: Option<Vec<u8>> = api
            .get_module(&at, module_id)
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err5)?;
        let f = convert::struct_to_json(&tag,f.unwrap(),module.unwrap()).map_err(runtime_error_into_rpc_err4).map_err(runtime_error_into_rpc_err6)?;
            let ff=serde_json::to_vec(&f).ok();
            println!("get_resources=result==={:?}=={:?}=",f,ff);
        let f=ff;
            Ok(f.map(Into::into))
       
    }


    fn get_resources2(
        &self,
        account_id: AccountId,
        tag: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<Bytes>> {
        let api = self.client.runtime_api();
        let att = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));
            let (tag_bcs,tag,module_id)=convert::parse_struct_tag_string(tag.into_vec()).unwrap();

        let f: Option<Vec<u8>> = api
            .get_resource(&att, account_id.clone(), tag_bcs)
            .map_err(runtime_error_into_rpc_err4)?
            .map_err(runtime_error_into_rpc_err5)?;
    let view=ApiStateView::new(self.client.clone(),account_id.clone(),at);
        use move_resource_viewer::MoveValueAnnotator;
     let annotator = move_resource_viewer::MoveValueAnnotator::new(&view);
use crate::move_types::{MoveResource};

        let f:MoveResource=   annotator
                                .view_resource(&tag, &f.unwrap())
                                .and_then(|result| {
                                    println!("=get_resources2===={:?}",result);
                                          result.try_into()
                                }).map_err(runtime_error_into_rpc_err5)?;

    //  let module: Option<Vec<u8>> = api
    //         .get_module(&at, module_id)
    //         .map_err(runtime_error_into_rpc_err4)?
    //         .map_err(runtime_error_into_rpc_err5)?;
        // let f = convert::struct_to_json(&tag,f.unwrap(),module.unwrap()).map_err(runtime_error_into_rpc_err4).map_err(runtime_error_into_rpc_err6)?;
            let ff=serde_json::to_vec(&f).ok();
            println!("get_resources2=result==={:?}=={:?}=",f,ff);
        let f=ff;
            Ok(f.map(Into::into))
       
    }
}
const RUNTIME_ERROR: i32 = 500;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
    CallError::Custom(ErrorObject::owned(
        RUNTIME_ERROR,
        "Error during requesting Runtime API",
        Some(format!("{:?}", err)),
    ))
    .into()
}

fn runtime_error_into_rpc_err2(err: impl std::fmt::Debug) -> JsonRpseeError {
    CallError::Custom(ErrorObject::owned(
        RUNTIME_ERROR,
        "Error during publishing module for estimation",
        Some(format!("{:?}", err)),
    ))
    .into()
}

fn runtime_error_into_rpc_err3(err: impl std::fmt::Debug) -> JsonRpseeError {
    CallError::Custom(ErrorObject::owned(
        RUNTIME_ERROR,
        "Error during script execution  for estimation",
        Some(format!("{:?}", err)),
    ))
    .into()
}

fn runtime_error_into_rpc_err4(err: impl std::fmt::Debug) -> JsonRpseeError {
    CallError::Custom(ErrorObject::owned(
        RUNTIME_ERROR,
        "ABI error",
        Some(format!("{:?}", err)),
    ))
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

fn runtime_error_into_rpc_err6(err: impl std::fmt::Debug) -> JsonRpseeError {
    CallError::Custom(ErrorObject::owned(
        RUNTIME_ERROR,
        "Error from struct tag json",
        Some(format!("{:?}", err)),
    ))
    .into()
}