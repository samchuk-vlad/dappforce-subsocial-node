use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

use pallet_utils::{Content, from_bool_to_option};

use crate::{Module, Profile, SocialAccount, Trait};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatProfile<AccountId, BlockNumber> {
    pub created_by: AccountId,
    pub created_at_block: BlockNumber,
    pub created_at_time: u64,
    pub updated_by: Option<AccountId>,
    pub updated_at_block: Option<BlockNumber>,
    pub updated_at_time: Option<u64>,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: Content,
    pub is_ipfs_content: Option<bool>,
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatSocialAccount<AccountId, BlockNumber> {
    pub id: AccountId,
    pub followers_count: u32,
    pub following_accounts_count: u16,
    pub following_spaces_count: u16,
    pub reputation: u32,
    pub profile: Option<FlatProfile<AccountId, BlockNumber>>,
}

impl<T: Trait> From<Profile<T>> for FlatProfile<T::AccountId, T::BlockNumber> {
    fn from(from: Profile<T>) -> Self {
        let Profile { created, updated, content } = from;

        Self {
            created_by: created.account,
            created_at_block: created.block,
            created_at_time: created.time.saturated_into::<u64>(),
            updated_by: updated.clone().map(|value| value.account),
            updated_at_block: updated.clone().map(|value| value.block),
            updated_at_time: updated.map(|value| value.time.saturated_into::<u64>()),
            content: content.clone(),
            is_ipfs_content: from_bool_to_option(content.is_ipfs()),
        }
    }
}

impl<T: Trait> From<SocialAccount<T>> for FlatSocialAccount<T::AccountId, T::BlockNumber> {
    fn from(from: SocialAccount<T>) -> Self {
        let SocialAccount {
            followers_count, following_accounts_count, following_spaces_count, reputation, profile
        } = from;

        Self {
            id: T::AccountId::default(),
            followers_count,
            following_accounts_count,
            following_spaces_count,
            reputation,
            profile: profile.map(|profile| profile.into()),
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_social_accounts_by_ids(
        account_ids: Vec<T::AccountId>
    ) -> Vec<FlatSocialAccount<T::AccountId, T::BlockNumber>> {
        account_ids.iter()
                   .filter_map(|account| {
                       Self::social_account_by_id(account)
                           .map(|social_account| social_account.into())
                           .map(|mut flat_social_account: FlatSocialAccount<T::AccountId, T::BlockNumber>| {
                               flat_social_account.id = account.clone();
                               flat_social_account
                           })
                   })
                   .collect()
    }
}