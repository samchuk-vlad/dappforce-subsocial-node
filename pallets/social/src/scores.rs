use super::*;

use frame_support::{dispatch::{DispatchResult}};

impl<T: Trait> Module<T> {

    pub fn change_post_score_by_extension(account: T::AccountId, post: &mut Post<T>, action: ScoringAction) -> DispatchResult {
        if post.is_comment() {
            Self::change_comment_score(account, post, action)?;
        } else {
            Self::change_post_score(account, post, action)?;
        }

        Ok(())
    }

    fn change_post_score(account: T::AccountId, post: &mut Post<T>, action: ScoringAction) -> DispatchResult {
        let social_account = Self::get_or_new_social_account(account.clone());
        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        let post_id = post.id;
        Post::<T>::ensure_post_stored(post_id)?;
        ensure!(!post.is_comment(), Error::<T>::PostIsAComment);

        if let Some(post_space_id) = post.space_id {
            let mut space = Self::space_by_id(post_space_id).ok_or(Error::<T>::SpaceNotFound)?;

            if post.created.account != account {
                if let Some(score_diff) = Self::post_score_by_account((account.clone(), post_id, action)) {
                    let reputation_diff = Self::account_reputation_diff_by_account((account.clone(), post.created.account.clone(), action))
                      .ok_or(Error::<T>::ReputationDiffNotFound)?;

                    post.score = post.score.checked_add(-(score_diff as i32)).ok_or(Error::<T>::OutOfBoundsRevertingPostScore)?;
                    space.score = space.score.checked_add(-(score_diff as i32)).ok_or(Error::<T>::OutOfBoundsRevertingSpaceScore)?;
                    Self::change_social_account_reputation(post.created.account.clone(), account.clone(), -reputation_diff, action)?;
                    <PostScoreByAccount<T>>::remove((account, post_id, action));
                } else {
                    match action {
                        ScoringAction::UpvotePost => {
                            if Self::post_score_by_account((account.clone(), post_id, ScoringAction::DownvotePost)).is_some() {
                                Self::change_post_score(account.clone(), post, ScoringAction::DownvotePost)?;
                            }
                        },
                        ScoringAction::DownvotePost => {
                            if Self::post_score_by_account((account.clone(), post_id, ScoringAction::UpvotePost)).is_some() {
                                Self::change_post_score(account.clone(), post, ScoringAction::UpvotePost)?;
                            }
                        },
                        _ => (),
                    }
                    let score_diff = Self::get_score_diff(social_account.reputation, action);
                    post.score = post.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingPostScore)?;
                    space.score = space.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingSpaceScore)?;
                    Self::change_social_account_reputation(post.created.account.clone(), account.clone(), score_diff, action)?;
                    <PostScoreByAccount<T>>::insert((account, post_id, action), score_diff);
                }

                <PostById<T>>::insert(post_id, post.clone());
                <SpaceById<T>>::insert(post_space_id, space);
            }
        }

        Ok(())
    }

    fn change_comment_score(account: T::AccountId, comment: &mut Post<T>, action: ScoringAction) -> DispatchResult {
        let social_account = Self::get_or_new_social_account(account.clone());
        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        let comment_id = comment.id;
        Post::<T>::ensure_post_stored(comment_id)?;
        let comment_ext = comment.get_comment_ext()?;

        ensure!(comment.is_comment(), Error::<T>::PostIsNotAComment);

        if comment.created.account != account {
            if let Some(score_diff) = Self::post_score_by_account((account.clone(), comment_id, action)) {
                let reputation_diff = Self::account_reputation_diff_by_account((account.clone(), comment.created.account.clone(), action))
                  .ok_or(Error::<T>::ReputationDiffNotFound)?;

                comment.score = comment.score.checked_add(-(score_diff as i32)).ok_or(Error::<T>::OutOfBoundsRevertingCommentScore)?;
                Self::change_social_account_reputation(comment.created.account.clone(), account.clone(), -reputation_diff, action)?;
                <PostScoreByAccount<T>>::remove((account, comment_id, action));
            } else {
                match action {
                    ScoringAction::UpvoteComment => {
                        if Self::post_score_by_account((account.clone(), comment_id, ScoringAction::DownvoteComment)).is_some() {
                            Self::change_comment_score(account.clone(), comment, ScoringAction::DownvoteComment)?;
                        }
                    },
                    ScoringAction::DownvoteComment => {
                        if Self::post_score_by_account((account.clone(), comment_id, ScoringAction::UpvoteComment)).is_some() {
                            Self::change_comment_score(account.clone(), comment, ScoringAction::UpvoteComment)?;
                        }
                    },
                    ScoringAction::CreateComment => {
                        let post = &mut (Self::post_by_id(comment_ext.root_post_id).ok_or(Error::<T>::PostNotFound)?);
                        Self::change_post_score(account.clone(), post, action)?;
                    }
                    _ => (),
                }
                let score_diff = Self::get_score_diff(social_account.reputation, action);
                comment.score = comment.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingCommentScore)?;
                Self::change_social_account_reputation(comment.created.account.clone(), account.clone(), score_diff, action)?;
                <PostScoreByAccount<T>>::insert((account, comment_id, action), score_diff);
            }
            <PostById<T>>::insert(comment_id, comment.clone());
        }

        Ok(())
    }

    pub fn change_social_account_reputation(
        account: T::AccountId,
        scorer: T::AccountId,
        mut score_diff: i16,
        action: ScoringAction
    ) -> DispatchResult {

        let mut social_account = Self::get_or_new_social_account(account.clone());

        if social_account.reputation as i64 + score_diff as i64 <= 1 {
            social_account.reputation = 1;
            score_diff = 0;
        }

        if score_diff < 0 {
            social_account.reputation = social_account.reputation.checked_sub(-score_diff as u32)
              .ok_or(Error::<T>::OutOfBoundsUpdatingAccountReputation)?;
        } else {
            social_account.reputation = social_account.reputation.checked_add(score_diff as u32)
              .ok_or(Error::<T>::OutOfBoundsUpdatingAccountReputation)?;
        }

        if Self::account_reputation_diff_by_account((scorer.clone(), account.clone(), action)).is_some() {
            <AccountReputationDiffByAccount<T>>::remove((scorer, account.clone(), action));
        } else {
            <AccountReputationDiffByAccount<T>>::insert((scorer, account.clone(), action), score_diff);
        }

        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        Self::deposit_event(RawEvent::AccountReputationChanged(account, action, social_account.reputation));

        Ok(())
    }

    pub fn get_score_diff(reputation: u32, action: ScoringAction) -> i16 {
        let r = Self::log_2(reputation);
        let d = (reputation - (2 as u32).pow(r)) * 100 / (2 as u32).pow(r);
        let score_diff = ((r + 1) * 100 + d) / 100;

        score_diff as i16 * Self::weight_of_scoring_action(action)
    }

    pub fn weight_of_scoring_action(action: ScoringAction) -> i16 {
        match action {
            ScoringAction::UpvotePost => T::UpvotePostActionWeight::get(),
            ScoringAction::DownvotePost => T::DownvotePostActionWeight::get(),
            ScoringAction::SharePost => T::SharePostActionWeight::get(),
            ScoringAction::CreateComment => T::CreateCommentActionWeight::get(),
            ScoringAction::UpvoteComment => T::UpvoteCommentActionWeight::get(),
            ScoringAction::DownvoteComment => T::DownvoteCommentActionWeight::get(),
            ScoringAction::ShareComment => T::ShareCommentActionWeight::get(),
            ScoringAction::FollowSpace => T::FollowSpaceActionWeight::get(),
            ScoringAction::FollowAccount => T::FollowAccountActionWeight::get(),
        }
    }

    fn num_bits<P>() -> usize { sp_std::mem::size_of::<P>() * 8 }

    pub fn log_2(x: u32) -> u32 {
        assert!(x > 0);
        Self::num_bits::<u32>() as u32 - x.leading_zeros() - 1
    }
}