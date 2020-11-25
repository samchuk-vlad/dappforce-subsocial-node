//! RPC interface for the transaction payment module.

use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::generic::BlockId;
use sp_runtime::traits::Block as BlockT;

use pallet_utils::SpaceId;
use spaces_runtime_api::SpacesApi as SpacesRuntimeApi;
use spaces_runtime_api::SpaceSerializable;

#[rpc]
pub trait SpacesApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "spaces_getLastSpaceId")]
    fn get_last_space_id(&self, at: Option<BlockHash>) -> Result<SpaceId>;

    #[rpc(name = "spaces_findPublicSpaces")]
    fn find_public_spaces(
        &self,
        at: Option<BlockHash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<SpaceSerializable<AccountId, BlockNumber>>>;

    #[rpc(name = "spaces_findUnlistedSpaces")]
    fn find_unlisted_spaces(
        &self,
        at: Option<BlockHash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<SpaceSerializable<AccountId, BlockNumber>>>;
}

pub struct Spaces<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Spaces<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, BlockNumber> SpacesApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
for Spaces<C, Block>
    where
        Block: BlockT,
        AccountId: Codec,
        BlockNumber: Codec,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: SpacesRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_last_space_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<SpaceId> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.get_last_space_id(&at);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn find_public_spaces(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<SpaceSerializable<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.find_public_spaces(&at, offset, limit);
        runtime_api_result.map_err(|e| RpcError {
            // TODO: research on error codes and change a value
            code: ErrorCode::ServerError(9876), // No real reason for this value
            // TODO: change error message (?use errors macro)
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn find_unlisted_spaces(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<SpaceSerializable<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.find_unlisted_spaces(&at, offset, limit);
        runtime_api_result.map_err(|e| RpcError {
            // TODO: research on error codes and change a value
            code: ErrorCode::ServerError(9876), // No real reason for this value
            // TODO: change error message (?use errors macro)
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
