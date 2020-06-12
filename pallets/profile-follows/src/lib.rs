#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use sp_std::prelude::*;
use system::ensure_signed;

use pallet_profiles::{Module as Profiles, SocialAccountById};
use pallet_utils::vec_remove_on;

// mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_profiles::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub AccountFollowers get(fn account_followers): map T::AccountId => Vec<T::AccountId>;
        pub AccountFollowedByAccount get(fn account_followed_by_account): map (T::AccountId, T::AccountId) => bool;
        pub AccountsFollowedByAccount get(fn accounts_followed_by_account): map T::AccountId => Vec<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        AccountFollowed(/* follower */ AccountId, /* following */ AccountId),
        AccountUnfollowed(/* follower */ AccountId, /* unfollowing */ AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Follower social account was not found by id.
        FollowerAccountNotFound,
        /// Social account that is being followed was not found by id.
        FollowedAccountNotFound,

        /// Account can not follow itself.
        AccountCannotFollowItself,
        /// Account can not unfollow itself.
        AccountCannotUnfollowItself,
        
        /// Account (Alice) is already a follower of another account (Bob).
        AlreadyAccountFollower,
        /// Account (Alice) is not a follower of another account (Bob).
        NotAccountFollower,
        
        /// Overflow caused following account.
        FollowAccountOverflow,
        /// Underflow caused unfollowing account.
        UnfollowAccountUnderflow,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing events
    fn deposit_event() = default;

    pub fn follow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotFollowItself);
      ensure!(!<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())),
        Error::<T>::AlreadyAccountFollower);

      let mut follower_account = Profiles::get_or_new_social_account(follower.clone());
      let mut followed_account = Profiles::get_or_new_social_account(account.clone());

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_add(1).ok_or(Error::<T>::FollowAccountOverflow)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_add(1).ok_or(Error::<T>::FollowAccountOverflow)?;

      // TODO old change_social_account_reputation
      // Self::change_social_account_reputation(
      //   account.clone(),
      //   follower.clone(),
      //   Self::score_diff_for_action(follower_account.reputation, ScoringAction::FollowAccount),
      //   ScoringAction::FollowAccount
      // )?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |ids| ids.push(follower.clone()));
      <AccountFollowedByAccount<T>>::insert((follower.clone(), account.clone()), true);

      Self::deposit_event(RawEvent::AccountFollowed(follower, account));

      // TODO new change_social_account_reputation
      // T::OnBeforeAccountFollowed::on_before_account_followed(...);
    }

    pub fn unfollow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotUnfollowItself);

      let mut follower_account = Profiles::social_account_by_id(follower.clone()).ok_or(Error::<T>::FollowerAccountNotFound)?;
      let mut followed_account = Profiles::social_account_by_id(account.clone()).ok_or(Error::<T>::FollowedAccountNotFound)?;

      ensure!(<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), Error::<T>::NotAccountFollower);

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_sub(1).ok_or(Error::<T>::UnfollowAccountUnderflow)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_sub(1).ok_or(Error::<T>::UnfollowAccountUnderflow)?;

      // TODO old change_social_account_reputation
      // let reputation_diff = Self::account_reputation_diff_by_account(
      //   (follower.clone(), account.clone(), ScoringAction::FollowAccount)
      // ).ok_or(Error::<T>::ReputationDiffNotFound)?;
      // Self::change_social_account_reputation(
      //   account.clone(),
      //   follower.clone(),
      //   reputation_diff,
      //   ScoringAction::FollowAccount
      // )?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |account_ids| vec_remove_on(account_ids, account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |account_ids| vec_remove_on(account_ids, follower.clone()));
      <AccountFollowedByAccount<T>>::remove((follower.clone(), account.clone()));

      Self::deposit_event(RawEvent::AccountUnfollowed(follower, account));

      // TODO new change_social_account_reputation
      // T::OnBeforeAccountUnfollowed::on_before_account_unfollowed(...);
    }
  }
}
