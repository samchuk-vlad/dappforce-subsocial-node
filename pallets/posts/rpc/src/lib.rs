use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use pallet_posts::rpc::FlatPost;
use pallet_utils::{PostId, SpaceId};
use posts_runtime_api::PostsApi as PostsRuntimeApi;

#[rpc]
pub trait PostsApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "posts_getPostsByIds")]
    fn get_posts_by_ids(
        &self,
        at: Option<BlockHash>,
        post_ids: Vec<PostId>,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getPublicPosts")]
    fn get_public_posts(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getUnlistedPosts")]
    fn get_unlisted_posts(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getReplyIdsByPostId")]
    fn get_reply_ids_by_post_id(
        &self,
        at: Option<BlockHash>,
        post_id: PostId,
    ) -> Result<Vec<PostId>>;

    /*#[rpc(name = "posts_getPostReplies")]
    fn get_post_replies(
        &self,
        at: Option<BlockHash>,
        post_id: PostId,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;*/

    #[rpc(name = "posts_getPostIdsBySpaceId")]
    fn get_post_ids_by_space_id(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
    ) -> Result<Vec<PostId>>;

    #[rpc(name = "posts_nextPostId")]
    fn get_next_post_id(&self, at: Option<BlockHash>) -> Result<PostId>;
}

pub struct Posts<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Posts<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, BlockNumber> PostsApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
for Posts<C, Block>
    where
        Block: BlockT,
        AccountId: Codec,
        BlockNumber: Codec,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: PostsRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_posts_by_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        post_ids: Vec<u64>,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_posts_by_ids(&at, post_ids);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_posts(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        space_id: u64,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_posts(&at, space_id, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_posts(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        space_id: u64,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_posts(&at, space_id, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_reply_ids_by_post_id(&self, at: Option<<Block as BlockT>::Hash>, post_id: u64) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_reply_ids_by_post_id(&at, post_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    /*fn get_post_replies(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        post_id: u64,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_post_replies(&at, post_id);
        runtime_api_result.map_err(map_rpc_error)
    }*/

    fn get_post_ids_by_space_id(&self, at: Option<<Block as BlockT>::Hash>, space_id: u64) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_post_ids_by_space_id(&at, space_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_next_post_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_next_post_id(&at);
        runtime_api_result.map_err(map_rpc_error)
    }
}

fn map_rpc_error(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(1),
        message: "An RPC error occurred".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
