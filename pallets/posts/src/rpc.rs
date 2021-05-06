use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::collections::{ btree_map::BTreeMap, btree_set::BTreeSet };
use sp_std::iter::FromIterator;
use sp_std::{vec, prelude::*};

use pallet_space_follows::Module as SpaceFollows;
use pallet_spaces::Module as Spaces;
use pallet_utils::{PostId, rpc::{FlatContent, FlatWhoAndWhen, ShouldSkip}, SpaceId};

use crate::{Module, Post, PostExtension, Trait};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatPostExtension {
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_regular_post: Option<bool>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_shared_post: Option<bool>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_comment: Option<bool>,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub root_post_id: Option<PostId>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub parent_post_id: Option<PostId>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub shared_post_id: Option<PostId>,
}

impl From<PostExtension> for FlatPostExtension {
    fn from(from: PostExtension) -> Self {
        let mut flat_extension = Self::default();

        match from {
            PostExtension::RegularPost => {
                flat_extension.is_regular_post = Some(true);
            }
            PostExtension::Comment(comment_ext) => {
                flat_extension.is_comment = Some(true);
                flat_extension.root_post_id = Some(comment_ext.root_post_id);
                flat_extension.parent_post_id = comment_ext.parent_id;
            }
            PostExtension::SharedPost(shared_post_id) => {
                flat_extension.is_shared_post = Some(true);
                flat_extension.shared_post_id = Some(shared_post_id);
            }
        }

        flat_extension
    }
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatPost<AccountId, BlockNumber> {
    pub id: PostId,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub who_and_when: FlatWhoAndWhen<AccountId, BlockNumber>,

    pub owner: AccountId,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub space_id: Option<SpaceId>,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: FlatContent,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_hidden: Option<bool>,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub extension: FlatPostExtension,

    pub visible_replies_count: u16,

    pub shares_count: u16,
    pub upvotes_count: u16,
    pub downvotes_count: u16,

    pub score: i32,
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ExtFilter {
    RegularPost,
    Comment,
    SharedPost
}

impl<T: Trait> From<Post<T>> for ExtFilter {
    fn from(from: Post<T>) -> Self {
        match from.extension {
            PostExtension::RegularPost => { Self::RegularPost }
            PostExtension::Comment(_) => { Self::Comment }
            PostExtension::SharedPost(_) => { Self::SharedPost }
        }
    }
}

impl<T: Trait> From<Post<T>> for FlatPost<T::AccountId, T::BlockNumber> {
    fn from(from: Post<T>) -> Self {
        let Post {
            id, created, updated, owner,
            extension, space_id, content, hidden, replies_count,
            hidden_replies_count, shares_count, upvotes_count, downvotes_count, score
        } = from;

        Self {
            id,
            who_and_when: (created, updated).into(),
            owner,
            space_id,
            content: content.into(),
            is_hidden: Some(hidden).filter(|value| *value),
            extension: extension.into(),
            visible_replies_count: replies_count.saturating_sub(hidden_replies_count),
            shares_count,
            upvotes_count,
            downvotes_count,
            score,
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_public_posts(
        ext_filter: Vec<ExtFilter>,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let not_filter = ext_filter.is_empty();
        let ext_filter_set: BTreeSet<_> = BTreeSet::from_iter(ext_filter.into_iter());

        let mut posts = Vec::new();
        let mut post_id = Self::get_next_post_id().saturating_sub(offset + 1);

        loop {
            if let Ok(post) = Self::require_post(post_id) {
                if post.is_public() &&
                    (not_filter || ext_filter_set.contains(&ExtFilter::from(post))) {
                    posts.push(post.into());
                }
            }

            if posts.len() >= limit as usize || post_id <= 1 { break; }
            post_id.saturating_sub(1);
        }

        posts
    }

    pub fn get_public_posts_by_ids(
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        Self::inner_get_posts_by_ids(post_ids, offset, limit, Some(|post: &Post<T>| post.is_public()))
    }

    pub fn get_posts_by_ids (
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        Self::inner_get_posts_by_ids(post_ids, offset, limit, None)
    }

    fn get_posts_slice_by_space_id<F: FnMut(&Post<T>) -> bool>(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
        compare_fn: F,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let post_ids: Vec<PostId> = Self::post_ids_by_space_id(space_id);

        Self::inner_get_posts_by_ids(post_ids, offset, limit, Some(compare_fn))
    }

    fn inner_get_posts_by_ids<F: FnMut(&Post<T>) -> bool>(
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
        compare_fn: Option<F>,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut posts = Vec::new();
        let is_compare_fn = compare_fn.is_some();

        let (_, offset_posts_ids) = post_ids.split_at(offset as usize);

        for post_id in offset_posts_ids.iter() {
            if let Ok(post) = Self::require_post(*post_id) {
                if is_compare_fn && compare_fn(&post) {
                    posts.push(post.into());
                }
            }

            if posts.len() >= limit as usize { break; }
        }

        posts
    }

    pub fn get_public_posts_by_space_id(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let public_space = Spaces::<T>::require_space(space_id).ok().filter(|space| space.is_public());
        if public_space.is_some() {
            return Self::get_posts_slice_by_space_id(space_id, offset, limit, |post| post.is_public());
        }

        vec![]
    }

    pub fn get_unlisted_posts_by_space_id(
        space_id: SpaceId,
        offset: u64,
        limit: u16,
    ) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let unlisted_space = Spaces::<T>::require_space(space_id).ok().filter(|space| !space.is_public());
        if unlisted_space.is_some() {
            return Self::get_posts_slice_by_space_id(space_id, offset, limit, |post| !post.is_public());
        }

        vec![]
    }

    pub fn get_reply_ids_by_post_id(post_id: PostId) -> Vec<PostId> {
        Self::try_get_post_replies_ids(post_id)
    }

    // TODO: replace with get_comment_tree
    //  - Additionally check comments depth
    /*pub fn get_post_replies(post_id: PostId) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let replies_ids = Self::try_get_post_replies_ids(post_id);
        Self::get_posts_by_ids(replies_ids)
    }*/

    pub fn get_comment_ids_tree(post_id: PostId) -> BTreeMap<PostId, Vec<PostId>> {
        BTreeMap::from_iter(Self::get_post_reply_ids_tree(post_id))
    }

    fn get_post_ids_by_space<F: FnMut(&Post<T>) -> bool>(space_id: SpaceId, mut filter: F) -> Vec<PostId> {
        Self::post_ids_by_space_id(space_id)
            .iter()
            .filter_map(Self::post_by_id)
            .filter(|post| filter(post))
            .map(|post| post.id)
            .collect()
    }

    pub fn get_public_post_ids_by_space_id(space_id: SpaceId) -> Vec<PostId> {
        let public_space = Spaces::<T>::require_space(space_id).ok().filter(|space| space.is_public());
        if public_space.is_some() {
            return Self::get_post_ids_by_space(space_id, |post| post.is_public());
        }

        vec![]
    }

    pub fn get_unlisted_post_ids_by_space_id(space_id: SpaceId) -> Vec<PostId> {
        let unlisted_space = Spaces::<T>::require_space(space_id).ok().filter(|space| !space.is_public());
        if unlisted_space.is_some() {
            return Self::get_post_ids_by_space(space_id, |post| !post.is_public());
        }

        vec![]
    }

    pub fn get_next_post_id() -> PostId {
        Self::next_post_id()
    }

    pub fn get_feed(account: T::AccountId, offset: u64, limit: u16) -> Vec<FlatPost<T::AccountId, T::BlockNumber>> {
        let mut post_ids: Vec<PostId> = SpaceFollows::<T>::spaces_followed_by_account(account)
            .iter()
            .flat_map(Self::post_ids_by_space_id)
            .collect();

        post_ids.sort_by(|a, b| b.cmp(a));

        Self::inner_get_posts_by_ids(post_ids, offset, limit, Some(|post: &Post<T>| post.is_public() && !post.is_comment()))
    }
}
