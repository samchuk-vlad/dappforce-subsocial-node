use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

use pallet_spaces::{Module as Spaces, Space};
use pallet_utils::{Content, from_bool_to_option, PostId, SpaceId};

use crate::{Module, Post, PostExtension, Trait};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatPost<AccountId, BlockNumber> {
    pub id: PostId,
    pub created_by: AccountId,
    pub created_at_block: BlockNumber,
    pub created_at_time: u64,
    pub updated_by: Option<AccountId>,
    pub updated_at_block: Option<BlockNumber>,
    pub updated_at_time: Option<u64>,

    pub owner: AccountId,

    pub space_id: Option<SpaceId>,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: Content,
    pub is_ipfs_content: Option<bool>,

    pub hidden: Option<bool>,

    #[cfg_attr(features = "std", serde(flatten))]
    pub extension: PostExtension,

    pub is_regular_post: Option<bool>,
    pub is_shared_post: Option<bool>,
    pub is_comment: Option<bool>,

    pub root_post_id: Option<PostId>,
    pub parent_post_id: Option<PostId>,
    pub shared_post_id: Option<PostId>,

    pub visible_replies_count: u16,

    pub shares_count: u16,
    pub upvotes_count: u16,
    pub downvotes_count: u16,

    pub score: i32,
}

impl<T: Trait> From<Post<T>> for FlatPost<T::AccountId, T::BlockNumber> {
    fn from(from: Post<T>) -> Self {
        let Post {
            id, created, updated, owner,
            extension, space_id, content, hidden, replies_count,
            hidden_replies_count, shares_count, upvotes_count, downvotes_count, score
        } = from.clone();

        let comment_ext = from.get_comment_ext().ok();

        Self {
            id,
            created_by: created.account,
            created_at_block: created.block,
            created_at_time: created.time.saturated_into::<u64>(),
            updated_by: updated.clone().map(|value| value.account),
            updated_at_block: updated.clone().map(|value| value.block),
            updated_at_time: updated.map(|value| value.time.saturated_into::<u64>()),
            owner,
            space_id,
            content: content.clone(),
            is_ipfs_content: from_bool_to_option(content.is_ipfs()),
            hidden: Some(hidden).filter(|value| *value == true),
            extension,
            is_regular_post: from_bool_to_option(from.is_regular_post()),
            is_shared_post: from_bool_to_option(from.is_sharing_post()),
            is_comment: from_bool_to_option(from.is_comment()),
            root_post_id: comment_ext.clone().map(|comment_ext| comment_ext.root_post_id),
            parent_post_id: comment_ext.and_then(|comment_ext| comment_ext.parent_id),
            shared_post_id: from.get_shared_post_id().ok(),
            visible_replies_count: replies_count.saturating_sub(hidden_replies_count),
            shares_count,
            upvotes_count,
            downvotes_count,
            score,
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_posts_by_ids(post_ids: Vec<PostId>) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut posts = Vec::new();
        for post_id in post_ids.iter() {
            if let Some(post) = Self::require_post(*post_id).ok() {
                posts.push(post.into());
            }
        }
        posts
    }

    fn get_posts_slice<F: FnMut(&Post<T>) -> bool>(
        space: &Space<T>,
        offset: u64,
        limit: u64,
        mut compare_fn: F,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut posts = Vec::new();

        let post_ids: Vec<PostId> = Self::post_ids_by_space_id(&space.id);
        let mut start_from = offset;
        let mut iterate_until = offset;
        let last_post_id = post_ids.len().saturating_sub(1) as u64;

        'outer: loop {
            iterate_until = iterate_until.saturating_add(limit);

            if start_from > last_post_id { break; }
            if iterate_until > last_post_id {
                iterate_until = last_post_id;
            }

            for post_id in start_from..=iterate_until {
                if let Some(post) = Self::require_post(post_id).ok() {
                    if compare_fn(&post) {
                        posts.push(post.into());
                        if posts.len() >= limit as usize { break 'outer; }
                    }
                }
            }
            start_from = iterate_until;
        }

        posts
    }

    pub fn get_public_posts(
        space_id: SpaceId,
        offset: u64,
        limit: u64,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let public_space = Spaces::<T>::require_space(space_id).ok().filter(|space| space.is_public());
        if let Some(space) = public_space {
            return Self::get_posts_slice(&space, offset, limit, |post| post.is_public());
        }

        vec![]
    }

    pub fn get_unlisted_posts(
        space_id: SpaceId,
        offset: u64,
        limit: u64,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let unlisted_space = Spaces::<T>::require_space(space_id).ok().filter(|space| !space.is_public());
        if let Some(space) = unlisted_space {
            return Self::get_posts_slice(&space, offset, limit, |post| !post.is_public());
        }

        vec![]
    }

    pub fn get_reply_ids_by_post_id(post_id: PostId) -> Vec<PostId> {
        Self::try_get_post_replies_ids(post_id)
    }

    /*pub fn get_post_replies(post_id: PostId) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let replies_ids = Self::try_get_post_replies_ids(post_id);
        Self::get_posts_by_ids(replies_ids)
    }*/

    pub fn get_post_ids_by_space_id(space_id: SpaceId) -> Vec<PostId> {
        Self::post_ids_by_space_id(space_id)
    }

    pub fn get_next_post_id() -> PostId {
        Self::next_post_id()
    }
}
