use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::SaturatedConversion;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::iter::FromIterator;
use sp_std::prelude::*;

use pallet_utils::PostId;

use crate::{Module, Reaction, ReactionId, ReactionKind, Trait};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatReaction<AccountId, BlockNumber> {
    pub id: ReactionId,
    pub created_by: AccountId,
    pub created_at_block: BlockNumber,
    pub created_at_time: u64,
    pub updated_by: Option<AccountId>,
    pub updated_at_block: Option<BlockNumber>,
    pub updated_at_time: Option<u64>,
    pub kind: ReactionKind,
}

impl<T: Trait> From<Reaction<T>> for FlatReaction<T::AccountId, T::BlockNumber> {
    fn from(from: Reaction<T>) -> Self {
        let Reaction { id, created, updated, kind } = from;

        Self {
            id,
            created_by: created.account,
            created_at_block: created.block,
            created_at_time: created.time.saturated_into::<u64>(),
            updated_by: updated.clone().map(|v| v.account),
            updated_at_block: updated.clone().map(|v| v.block),
            updated_at_time: updated.map(|v| v.time.saturated_into::<u64>()),
            kind,
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_reactions_by_ids(
        reaction_ids: Vec<ReactionId>
    ) -> Vec<FlatReaction<T::AccountId, T::BlockNumber>> {
        reaction_ids.iter()
                    .filter_map(|id| Self::require_reaction(*id).ok())
                    .map(|reaction| reaction.into())
                    .collect()
    }

    pub fn get_reactions_by_post_id(
        post_id: PostId,
        limit: u64,
        offset: u64,
    ) -> Vec<FlatReaction<T::AccountId, T::BlockNumber>> {
        let mut reactions = Vec::new();

        let reaction_ids: Vec<PostId> = Self::reaction_ids_by_post_id(&post_id);
        let mut start_from = offset;
        let mut iterate_until = offset;
        let last_post_id = reaction_ids.len().saturating_sub(1) as u64;

        'outer: loop {
            iterate_until = iterate_until.saturating_add(limit);

            if start_from > last_post_id { break; }
            if iterate_until > last_post_id {
                iterate_until = last_post_id;
            }

            for reaction_id in start_from..=iterate_until {
                if let Some(reaction) = Self::require_reaction(reaction_id).ok() {
                    reactions.push(reaction.into());
                    if reactions.len() >= limit as usize { break 'outer; }
                }
            }
            start_from = iterate_until;
        }

        reactions
    }

    pub fn get_reactions_by_account(
        account: T::AccountId,
        post_ids: Vec<PostId>,
    ) -> BTreeMap<PostId, FlatReaction<T::AccountId, T::BlockNumber>> {
        let reaction_zipped_with_post_id =
            post_ids.iter()
                    .filter_map(|post_id|
                        Some(*post_id).zip(
                            Option::from(Self::post_reaction_id_by_account((&account, post_id)))
                                .filter(|v| *v != 0)
                                .and_then(|reaction_id|
                                    Self::require_reaction(reaction_id).ok().map(|reaction| reaction.into())
                                )
                        )
                    );

        BTreeMap::from_iter(reaction_zipped_with_post_id)
    }
}