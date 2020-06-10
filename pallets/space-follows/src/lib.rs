#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure,
};
use sp_std::prelude::*;
use system::ensure_signed;

use df_traits::SpaceFollowsProvider;
use pallet_profiles::{Module as Profiles, SocialAccountById};
use pallet_spaces::{Module as Spaces, Space, SpaceById};
use pallet_utils::{SpaceId, vec_remove_on};

// mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_spaces::Trait
    + pallet_profiles::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Social account was not found by id.
        SocialAccountNotFound,
        /// Account is already a space follower.
        AlreadySpaceFollower,
        /// Account is not a space follower.
        NotSpaceFollower,
        /// Not allowed to follow a hidden space.
        CannotFollowHiddenSpace,
        /// Overflow caused by following a space.
        FollowSpaceOverflow,
        /// Underflow caused by unfollowing a space.
        UnfollowSpaceUnderflow,
    }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub SpaceFollowers get(fn space_followers): map SpaceId => Vec<T::AccountId>;
        pub SpaceFollowedByAccount get(fn space_followed_by_account): map (T::AccountId, SpaceId) => bool;
        pub SpacesFollowedByAccount get(fn spaces_followed_by_account): map T::AccountId => Vec<SpaceId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        SpaceFollowed(/* follower */ AccountId, /* following */ SpaceId),
        SpaceUnfollowed(/* follower */ AccountId, /* unfollowing */ SpaceId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing events
    fn deposit_event() = default;

    pub fn follow_space(origin, space_id: SpaceId) {
      let follower = ensure_signed(origin)?;

      let space = &mut Spaces::require_space(space_id)?;
      ensure!(!Self::space_followed_by_account((follower.clone(), space_id)), Error::<T>::AlreadySpaceFollower);
      ensure!(!space.hidden, Error::<T>::CannotFollowHiddenSpace);

      Self::add_space_follower(follower, space)?;
      <SpaceById<T>>::insert(space_id, space);

      // TODO new
      // T::SpaceFollowsHandler::on_space_followed(...);
    }

    pub fn unfollow_space(origin, space_id: SpaceId) {
      let follower = ensure_signed(origin)?;

      let space = &mut Spaces::require_space(space_id)?;
      ensure!(Self::space_followed_by_account((follower.clone(), space_id)), Error::<T>::NotSpaceFollower);

      space.followers_count = space.followers_count
        .checked_sub(1)
        .ok_or(Error::<T>::UnfollowSpaceUnderflow)?;

      let mut social_account = Profiles::social_account_by_id(follower.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
      social_account.following_spaces_count = social_account.following_spaces_count
        .checked_sub(1)
        .ok_or(Error::<T>::UnfollowSpaceUnderflow)?;

      // TODO old change_social_account_reputation
      // if space.created.account != follower {
      //   let author = space.created.account.clone();
      //   if let Some(score_diff) = Self::account_reputation_diff_by_account((follower.clone(), author.clone(), ScoringAction::FollowSpace)) {
      //     space.score = space.score.checked_sub(score_diff as i32).ok_or(Error::<T>::SpaceScoreUnderflow)?;
      //     Self::change_social_account_reputation(author, follower.clone(), -score_diff, ScoringAction::FollowSpace)?;
      //   }
      // }

      <SpacesFollowedByAccount<T>>::mutate(follower.clone(), |space_ids| vec_remove_on(space_ids, space_id));
      <SpaceFollowers<T>>::mutate(space_id, |account_ids| vec_remove_on(account_ids, follower.clone()));
      <SpaceFollowedByAccount<T>>::remove((follower.clone(), space_id));
      <SocialAccountById<T>>::insert(follower.clone(), social_account);
      <SpaceById<T>>::insert(space_id, space);

      Self::deposit_event(RawEvent::SpaceUnfollowed(follower, space_id));

      // TODO new change_social_account_reputation
      // T::SpaceFollowsHandler::on_space_unfollowed(...);
    }
  }
}

impl<T: Trait> Module<T> {
    pub fn add_space_follower(follower: T::AccountId, space: &mut Space<T>) -> DispatchResult {
        space.followers_count = space.followers_count
            .checked_add(1)
            .ok_or(Error::<T>::FollowSpaceOverflow)?;

        let mut social_account = Profiles::get_or_new_social_account(follower.clone());
        social_account.following_spaces_count = social_account.following_spaces_count
            .checked_add(1)
            .ok_or(Error::<T>::FollowSpaceOverflow)?;

        // TODO old change_social_account_reputation
        // if space.created.account != follower {
        //     let author = space.created.account.clone();
        //     let score_diff = Self::score_diff_for_action(social_account.reputation, ScoringAction::FollowSpace);
        //     space.score = space.score.checked_add(score_diff as i32).ok_or(Error::<T>::SpaceScoreOverflow)?;
        //     Self::change_social_account_reputation(author, follower.clone(), score_diff, ScoringAction::FollowSpace)?;
        // }

        let space_id = space.id;
        <SpacesFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(space_id));
        <SpaceFollowers<T>>::mutate(space_id, |ids| ids.push(follower.clone()));
        <SpaceFollowedByAccount<T>>::insert((follower.clone(), space_id), true);
        <SocialAccountById<T>>::insert(follower.clone(), social_account);

        Self::deposit_event(RawEvent::SpaceFollowed(follower, space_id));

        // TODO new change_social_account_reputation
        // T::SpaceFollows::on_space_followed(...);

        Ok(())
    }
}

impl<T: Trait> SpaceFollowsProvider for Module<T> {
    type AccountId = T::AccountId;

    fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool {
        Module::<T>::space_followed_by_account((account, space_id))
    }
}
