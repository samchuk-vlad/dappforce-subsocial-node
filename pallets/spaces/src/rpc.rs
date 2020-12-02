use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::SaturatedConversion;
use sp_std::vec::Vec;

use pallet_utils::{Content, SpaceId, from_bool_to_option};

use crate::{Module, Space, Trait};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct SpaceSerializable<AccountId, BlockNumber> {
    pub id: SpaceId,
    pub created_by: AccountId,
    pub created_at_block: BlockNumber,
    pub created_at_time: u64,
    pub updated_by: Option<AccountId>,
    pub updated_at_block: Option<BlockNumber>,
    pub updated_at_time: Option<u64>,

    pub owner: AccountId,

    pub parent_id: Option<SpaceId>,
    pub handle: Option<Vec<u8>>,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: Content,

    pub is_ipfs_content: Option<bool>,
    pub hidden: Option<bool>,

    pub posts_count: u32,
    pub hidden_posts_count: u32,
    pub followers_count: u32,

    pub score: i32,
}

impl<T: Trait> From<Space<T>> for SpaceSerializable<T::AccountId, T::BlockNumber> {
    fn from(from: Space<T>) -> Self {
        let Space {
            id, created, updated, owner,
            parent_id, handle, content, hidden, posts_count,
            hidden_posts_count, followers_count, score, ..
        } = from;

        Self {
            id,
            created_by: created.account,
            created_at_block: created.block,
            created_at_time: created.time.saturated_into::<u64>(),
            updated_by: updated.clone().map(|value| value.account),
            updated_at_block: updated.clone().map(|value| value.block),
            updated_at_time: updated.map(|value| value.time.saturated_into::<u64>()),
            owner,
            parent_id,
            handle,
            content: content.clone(),
            is_ipfs_content: from_bool_to_option(content.is_ipfs()),
            hidden: from_bool_to_option(hidden),
            posts_count,
            hidden_posts_count,
            followers_count,
            score,
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_spaces_by_ids(space_ids: Vec<SpaceId>) -> Vec<SpaceSerializable<T::AccountId, T::BlockNumber>> {
        let mut spaces = Vec::new();
        for space_id in space_ids.iter() {
            if let Some(space) = Self::require_space(*space_id).ok() {
                spaces.push(space.into());
            }
        }
        spaces
    }

    fn get_spaces_slice<F: FnMut(&Space<T>) -> bool>(
        offset: u64,
        limit: u64,
        mut compare_fn: F,
    ) -> Vec<SpaceSerializable<T::AccountId, T::BlockNumber>> {
        let mut start_from = offset;
        let mut iterate_until = offset;
        let last_space_id = Self::next_space_id().saturating_sub(1);

        let mut spaces = Vec::new();

        'outer: loop {
            iterate_until = iterate_until.saturating_add(limit);

            if start_from > last_space_id { break; }
            if iterate_until > last_space_id {
                iterate_until = last_space_id;
            }

            for space_id in start_from..=iterate_until {
                if let Some(space) = Self::require_space(space_id).ok() {
                    if compare_fn(&space) {
                        spaces.push(space.into());
                        if spaces.len() >= limit as usize { break 'outer; }
                    }
                }
            }
            start_from = iterate_until;
        }

        spaces
    }

    pub fn get_spaces(offset: u64, limit: u64) -> Vec<SpaceSerializable<T::AccountId, T::BlockNumber>> {
        Self::get_spaces_slice(offset, limit, |_| true)
    }

    pub fn get_public_spaces(offset: u64, limit: u64) -> Vec<SpaceSerializable<T::AccountId, T::BlockNumber>> {
        Self::get_spaces_slice(offset, limit, |space| space.is_public())
    }

    pub fn get_unlisted_spaces(offset: u64, limit: u64) -> Vec<SpaceSerializable<T::AccountId, T::BlockNumber>> {
        Self::get_spaces_slice(offset, limit, |space| !space.is_public())
    }

    pub fn get_space_id_by_handle(handle: Vec<u8>) -> Option<SpaceId> {
        Self::space_id_by_handle(handle)
    }

    pub fn get_space_by_handle(handle: Vec<u8>) -> Option<SpaceSerializable<T::AccountId, T::BlockNumber>> {
        Self::space_id_by_handle(handle)
            .and_then(|space_id| Self::require_space(space_id).ok())
            .map(|space| space.into())
    }

    pub fn get_space_ids_by_owner(owner: T::AccountId) -> Vec<SpaceId> {
        Self::space_ids_by_owner(owner)
    }
}