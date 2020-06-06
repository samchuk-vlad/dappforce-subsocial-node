use super::*;

use frame_support::dispatch::DispatchResult;

impl<T: Trait> Module<T> {

    pub fn add_space_follower(
        follower: T::AccountId,
        space: &mut Space<T>
    ) -> DispatchResult {

        let space_id = space.id;
        let mut social_account = Self::get_or_new_social_account(follower.clone());
        social_account.following_spaces_count = social_account.following_spaces_count
            .checked_add(1)
            .ok_or(Error::<T>::OverflowFollowingSpace)?;

        space.followers_count = space.followers_count.checked_add(1).ok_or(Error::<T>::OverflowFollowingSpace)?;
        if space.created.account != follower {
            let author = space.created.account.clone();
            let score_diff = Self::get_score_diff(social_account.reputation, ScoringAction::FollowSpace);
            space.score = space.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingSpaceScore)?;
            Self::change_social_account_reputation(author, follower.clone(), score_diff, ScoringAction::FollowSpace)?;
        }

        <SocialAccountById<T>>::insert(follower.clone(), social_account);
        <SpacesFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(space_id));
        <SpaceFollowers<T>>::mutate(space_id, |ids| ids.push(follower.clone()));
        <SpaceFollowedByAccount<T>>::insert((follower.clone(), space_id), true);

        Self::deposit_event(RawEvent::SpaceFollowed(follower, space_id));
        Ok(())
    }
}
