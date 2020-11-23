#![cfg_attr(not(feature = "std"), no_std)]
// #![allow(clippy::too_many_arguments)]

use codec::{Encode, Codec, Decode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use sp_std::vec::Vec;

use pallet_utils::{SpaceId};
use pallet_permissions::SpacePermissions;

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct SpaceInfo<AccountId> {
    pub id: SpaceId,
    // #[serde(flatten)]
    // pub created: WhoAndWhen,
    // #[serde(flatten)]
    // pub updated: Option<WhoAndWhen>,

    pub owner: AccountId,

    // Can be updated by the owner:
    pub parent_id: Option<SpaceId>,
    pub handle: Option<Vec<u8>>,
    // #[serde(flatten)]
    // pub content: Content,
    pub hidden: bool,

    pub posts_count: u32,
    pub hidden_posts_count: u32,
    pub followers_count: u32,

    pub score: i32,

    // #[serde(flatten)]
    pub permissions: Option<SpacePermissions>,
}

sp_api::decl_runtime_apis! {
    pub trait SpacesApi<AccountId> where
        AccountId: Codec
    {
        fn get_last_space_id() -> SpaceId;

        fn get_hidden_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId>;

        fn find_public_space_ids(offset: u64, limit: u64) -> Vec<SpaceId>;

        fn find_unlisted_space_ids(offset: u64, limit: u64) -> Vec<SpaceId>;

        // fn find_public_spaces(offset: u64, limit: u64) -> Vec<Space<T>>;

        fn find_struct(space_id: SpaceId) -> SpaceInfo<AccountId>;
    }
}
