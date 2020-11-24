#![cfg_attr(not(feature = "std"), no_std)]
// #![allow(clippy::too_many_arguments)]

use codec::{Encode, Codec, Decode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use sp_std::vec::Vec;
use sp_runtime::SaturatedConversion;

use pallet_utils::{SpaceId, Content, WhoAndWhen};
use pallet_permissions::SpacePermissions;

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct WhoAndWhenSerializable<AccountId, BlockNumber> {
    pub account: AccountId,
    pub block: BlockNumber,
    pub time: u64,
}

impl<T: pallet_utils::Trait> From<WhoAndWhen<T>> for WhoAndWhenSerializable<T::AccountId, T::BlockNumber> {
    fn from(from: WhoAndWhen<T>) -> Self {
        Self {
            account: from.account,
            block: from.block,
            time: from.time.saturated_into::<u64>()
        }
    }
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct SpaceSerializable<AccountId, BlockNumber> {
    pub id: SpaceId,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub created: WhoAndWhenSerializable<AccountId, BlockNumber>,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub updated: Option<WhoAndWhenSerializable<AccountId, BlockNumber>>,

    pub owner: AccountId,

    // Can be updated by the owner:
    pub parent_id: Option<SpaceId>,
    pub handle: Option<Vec<u8>>,
    pub content: Content,
    pub hidden: bool,

    pub posts_count: u32,
    pub hidden_posts_count: u32,
    pub followers_count: u32,

    pub score: i32,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub permissions: Option<SpacePermissions>,
}

sp_api::decl_runtime_apis! {
    pub trait SpacesApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec
    {
        fn get_last_space_id() -> SpaceId;

        fn get_hidden_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId>;

        fn find_public_space_ids(offset: u64, limit: u64) -> Vec<SpaceId>;

        fn find_unlisted_space_ids(offset: u64, limit: u64) -> Vec<SpaceId>;

        // fn find_public_spaces(offset: u64, limit: u64) -> Vec<Space<T>>;

        fn find_struct(space_id: SpaceId) -> SpaceSerializable<AccountId, BlockNumber>;
    }
}
