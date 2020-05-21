use super::*;

use frame_support::{dispatch::{DispatchResult, DispatchError}};

impl<T: Trait> Module<T> {

    pub fn ensure_blog_exists(blog_id: BlogId) -> DispatchResult {
        ensure!(<BlogById<T>>::exists(blog_id), Error::<T>::BlogNotFound);
        Ok(())
    }

    // TODO: maybe don't add reaction in storage before checks in 'create_reaction' are done?
    pub fn new_reaction(account: T::AccountId, kind: ReactionKind) -> ReactionId {
        let reaction_id = Self::next_reaction_id();
        let new_reaction: Reaction<T> = Reaction {
            id: reaction_id,
            created: WhoAndWhen::<T>::new(account),
            updated: None,
            kind
        };

        <ReactionById<T>>::insert(reaction_id, new_reaction);
        NextReactionId::mutate(|n| { *n += 1; });

        reaction_id
    }

    pub fn add_blog_follower(
        follower: T::AccountId,
        blog: &mut Blog<T>
    ) -> DispatchResult {

        let blog_id = blog.id;
        let mut social_account = Self::get_or_new_social_account(follower.clone());
        social_account.following_blogs_count = social_account.following_blogs_count
            .checked_add(1)
            .ok_or(Error::<T>::OverflowFollowingBlog)?;

        blog.followers_count = blog.followers_count.checked_add(1).ok_or(Error::<T>::OverflowFollowingBlog)?;
        if blog.created.account != follower {
            let author = blog.created.account.clone();
            let score_diff = Self::get_score_diff(social_account.reputation, ScoringAction::FollowBlog);
            blog.score = blog.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingBlogScore)?;
            Self::change_social_account_reputation(author, follower.clone(), score_diff, ScoringAction::FollowBlog)?;
        }

        <SocialAccountById<T>>::insert(follower.clone(), social_account);
        <BlogsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(blog_id));
        <BlogFollowers<T>>::mutate(blog_id, |ids| ids.push(follower.clone()));
        <BlogFollowedByAccount<T>>::insert((follower.clone(), blog_id), true);

        Self::deposit_event(RawEvent::BlogFollowed(follower, blog_id));
        Ok(())
    }

    pub fn get_or_new_social_account(account: T::AccountId) -> SocialAccount<T> {
        if let Some(social_account) = Self::social_account_by_id(account) {
            social_account
        } else {
            SocialAccount {
                followers_count: 0,
                following_accounts_count: 0,
                following_blogs_count: 0,
                reputation: 1,
                profile: None
            }
        }
    }

    pub fn vec_remove_on<F: PartialEq>(vector: &mut Vec<F>, element: F) {
        if let Some(index) = vector.iter().position(|x| *x == element) {
            vector.swap_remove(index);
        }
    }

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
        ensure!(<PostById<T>>::exists(post_id), Error::<T>::PostNotFound);
        ensure!(!post.is_comment(), Error::<T>::PostIsAComment);

        if let Some(post_blog_id) = post.blog_id {
            let mut blog = Self::blog_by_id(post_blog_id).ok_or(Error::<T>::BlogNotFound)?;

            if post.created.account != account {
                if let Some(score_diff) = Self::post_score_by_account((account.clone(), post_id, action)) {
                    let reputation_diff = Self::account_reputation_diff_by_account((account.clone(), post.created.account.clone(), action)).ok_or(Error::<T>::ReputationDiffNotFound)?;
                    post.score = post.score.checked_add(-(score_diff as i32)).ok_or(Error::<T>::OutOfBoundsRevertingPostScore)?;
                    blog.score = blog.score.checked_add(-(score_diff as i32)).ok_or(Error::<T>::OutOfBoundsRevertingBlogScore)?;
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
                    blog.score = blog.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingBlogScore)?;
                    Self::change_social_account_reputation(post.created.account.clone(), account.clone(), score_diff, action)?;
                    <PostScoreByAccount<T>>::insert((account, post_id, action), score_diff);
                }

                <PostById<T>>::insert(post_id, post.clone());
                <BlogById<T>>::insert(post_blog_id, blog);
            }
        }

        Ok(())
    }

    fn change_comment_score(account: T::AccountId, comment: &mut Post<T>, action: ScoringAction) -> DispatchResult {
        let social_account = Self::get_or_new_social_account(account.clone());
        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        let comment_id = comment.id;
        ensure!(<PostById<T>>::exists(comment_id), Error::<T>::PostNotFound);
        let comment_ext = comment.get_comment_ext()?;

        ensure!(comment.is_comment(), Error::<T>::PostIsNotAComment);

        if comment.created.account != account {
            if let Some(score_diff) = Self::post_score_by_account((account.clone(), comment_id, action)) {
                let reputation_diff = Self::account_reputation_diff_by_account((account.clone(), comment.created.account.clone(), action)).ok_or(Error::<T>::ReputationDiffNotFound)?;
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

    pub fn change_social_account_reputation(account: T::AccountId, scorer: T::AccountId, mut score_diff: i16, action: ScoringAction) -> DispatchResult {
        let mut social_account = Self::get_or_new_social_account(account.clone());

        if social_account.reputation as i64 + score_diff as i64 <= 1 {
            social_account.reputation = 1;
            score_diff = 0;
        }

        if score_diff < 0 {
            social_account.reputation = social_account.reputation.checked_sub(-score_diff as u32).ok_or(Error::<T>::OutOfBoundsUpdatingAccountReputation)?;
        } else {
            social_account.reputation = social_account.reputation.checked_add(score_diff as u32).ok_or(Error::<T>::OutOfBoundsUpdatingAccountReputation)?;
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

    // TODO write unit tests for this method.
    pub fn weight_of_scoring_action(action: ScoringAction) -> i16 {
        match action {
            ScoringAction::UpvotePost => T::UpvotePostActionWeight::get(),
            ScoringAction::DownvotePost => T::DownvotePostActionWeight::get(),
            ScoringAction::SharePost => T::SharePostActionWeight::get(),
            ScoringAction::CreateComment => T::CreateCommentActionWeight::get(),
            ScoringAction::UpvoteComment => T::UpvoteCommentActionWeight::get(),
            ScoringAction::DownvoteComment => T::DownvoteCommentActionWeight::get(),
            ScoringAction::ShareComment => T::ShareCommentActionWeight::get(),
            ScoringAction::FollowBlog => T::FollowBlogActionWeight::get(),
            ScoringAction::FollowAccount => T::FollowAccountActionWeight::get(),
        }
    }

    fn num_bits<P>() -> usize { sp_std::mem::size_of::<P>() * 8 }

    pub fn log_2(x: u32) -> u32 {
        assert!(x > 0);
        Self::num_bits::<u32>() as u32 - x.leading_zeros() - 1
    }

    pub fn is_username_valid(username: Vec<u8>) -> DispatchResult {
        ensure!(Self::account_by_profile_username(username.clone()).is_none(), Error::<T>::UsernameIsBusy);
        ensure!(username.len() >= T::MinUsernameLen::get() as usize, Error::<T>::UsernameIsTooShort);
        ensure!(username.len() <= T::MaxUsernameLen::get() as usize, Error::<T>::UsernameIsTooLong);
        ensure!(username.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::UsernameIsNotAlphanumeric);

        Ok(())
    }

    pub fn is_ipfs_hash_valid(ipfs_hash: Vec<u8>) -> DispatchResult {
        ensure!(ipfs_hash.len() == T::IpfsHashLen::get() as usize, Error::<T>::IpfsIsIncorrect);

        Ok(())
    }

    pub fn share_post(account: T::AccountId, original_post: &mut Post<T>, shared_post_id: PostId) -> DispatchResult {
        original_post.shares_count = original_post.shares_count.checked_add(1).ok_or(Error::<T>::OverflowTotalShares)?;

        let original_post_id = original_post.id;

        let mut shares_count = Self::post_shares_by_account((account.clone(), original_post_id));
        shares_count = shares_count.checked_add(1).ok_or(Error::<T>::OverflowPostShares)?;

        if shares_count == 1 {
            Self::change_post_score_by_extension(account.clone(), original_post, {
                if original_post.is_comment() { ScoringAction::ShareComment } else {ScoringAction::SharePost}
            })?;
        }

        <PostById<T>>::insert(original_post_id, original_post.clone());
        <PostSharesByAccount<T>>::insert((account.clone(), original_post_id), shares_count);
        SharedPostIdsByOriginalPostId::mutate(original_post_id, |ids| ids.push(shared_post_id));

        Self::deposit_event(RawEvent::PostShared(account, original_post_id));
        Ok(())
    }

    pub fn scoring_action_by_post_extension(extension: PostExtension, reaction_kind: ReactionKind, reverse: bool) -> ScoringAction {
        let scoring_action;

        match extension {
            PostExtension::RegularPost | PostExtension::SharedPost(_) => match reaction_kind {
                ReactionKind::Upvote => scoring_action = if reverse {ScoringAction::DownvotePost} else {ScoringAction::UpvotePost},
                ReactionKind::Downvote => scoring_action = if reverse {ScoringAction::UpvotePost} else {ScoringAction::DownvotePost},
            },
            PostExtension::Comment(_) => match reaction_kind {
                ReactionKind::Upvote => scoring_action = if reverse {ScoringAction::DownvoteComment} else {ScoringAction::UpvoteComment},
                ReactionKind::Downvote => scoring_action = if reverse {ScoringAction::UpvoteComment} else {ScoringAction::DownvoteComment},
            },
        }

        scoring_action
    }
    
    fn is_valid_handle_char(c: u8) -> bool {
        match c {
            b'0'..=b'9' | b'a'..=b'z' | b'_' => true,
            _ => false,
        }
    }

    pub fn lowercase_and_validate_a_handle(mut handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {
        handle = handle.to_ascii_lowercase();

        ensure!(Self::blog_id_by_handle(handle.clone()).is_none(), Error::<T>::HandleIsNotUnique);

        ensure!(handle.len() >= T::MinHandleLen::get() as usize, Error::<T>::HandleIsTooShort);
        ensure!(handle.len() <= T::MaxHandleLen::get() as usize, Error::<T>::HandleIsTooLong);

        ensure!(handle.iter().all(|&x| Self::is_valid_handle_char(x)), Error::<T>::HandleContainsInvalidChars);

        Ok(handle)
    }

    pub fn is_root_post_hidden(post_id: PostId) -> Result<bool, DispatchError> {
        let post = Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?;
        let root_post = post.get_root_post()?;
        Ok(root_post.hidden)
    }

    pub fn get_ancestors(post_id: PostId) -> Vec<Post<T>> {
        let mut ancestors: Vec<Post<T>> = Vec::new();

        if let Some(post) = Self::post_by_id(post_id) {
            ancestors.push(post.clone());
            if let Some(parent_id) = post.get_comment_ext().ok().unwrap().parent_id {
                ancestors.extend(Self::get_ancestors(parent_id).iter().cloned());
            }
        }

        ancestors
    }
}

impl<T: Trait> Blog<T> {
    pub fn ensure_blog_owner(&self, who: T::AccountId) -> DispatchResult {
        ensure!(self.owner == who, Error::<T>::NotABlogOwner);
        Ok(())
    }

    pub fn increment_posts_count(&mut self) -> DispatchResult {
        self.posts_count = self.posts_count.checked_add(1).ok_or(Error::<T>::OverflowAddingPostOnBlog)?;
        Ok(())
    }
}

impl<T: Trait> Post<T> {
    pub fn create(id: PostId, created_by: T::AccountId, blog_id_opt: Option<BlogId>, extension: PostExtension, ipfs_hash: Vec<u8>) -> Self {
        Post {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            hidden: false,
            blog_id: blog_id_opt,
            extension,
            ipfs_hash,
            edit_history: vec![],
            direct_replies_count: 0,
            total_replies_count: 0,
            shares_count: 0,
            upvotes_count: 0,
            downvotes_count: 0,
            score: 0,
        }
    }

    pub fn is_comment(&self) -> bool {
        match self.extension {
            PostExtension::Comment(_) => true,
            _ => false,
        }
    }

    pub fn is_shared_post(&self) -> bool {
        match self.extension {
            PostExtension::SharedPost(_) => true,
            _ => false,
        }
    }

    pub fn get_comment_ext(&self) -> Result<CommentExt, DispatchError> {
        match self.extension {
            PostExtension::Comment(comment_ext) => Ok(comment_ext),
            _ => Err(Error::<T>::PostIsNotAComment.into())
        }
    }

    pub fn get_root_post(&self) -> Result<Post<T>, DispatchError> {
        match self.extension {
            PostExtension::RegularPost | PostExtension::SharedPost(_) => Ok(self.clone()),
            PostExtension::Comment(comment_ext) => Module::post_by_id(comment_ext.root_post_id).ok_or_else(|| Error::<T>::PostNotFound.into()),
        }
    }

    pub fn get_blog(&self) -> Result<Blog<T>, DispatchError> {
        let root_post = self.get_root_post()?;

        let blog_id = root_post.blog_id.ok_or(Error::<T>::BlogIdIsUndefined)?;
        let blog = Module::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?;

        Ok(blog)
    }
}
