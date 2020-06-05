use super::*;

use frame_support::dispatch::DispatchResult;
use df_traits::{SpaceForRolesProvider, SpaceForRoles};

impl<T: Trait> Module<T> {

    // FIXME: don't add reaction in storage before checks in 'create_reaction' are done
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

    pub fn get_or_new_social_account(account: T::AccountId) -> SocialAccount<T> {
        if let Some(social_account) = Self::social_account_by_id(account) {
            social_account
        } else {
            SocialAccount {
                followers_count: 0,
                following_accounts_count: 0,
                following_spaces_count: 0,
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

    pub fn is_username_valid(username: Vec<u8>) -> DispatchResult {
        ensure!(Self::account_by_profile_username(username.clone()).is_none(), Error::<T>::UsernameIsBusy);
        ensure!(username.len() >= T::MinUsernameLen::get() as usize, Error::<T>::UsernameIsTooShort);
        ensure!(username.len() <= T::MaxUsernameLen::get() as usize, Error::<T>::UsernameIsTooLong);
        ensure!(username.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::UsernameIsNotAlphanumeric);

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

        ensure!(Self::space_id_by_handle(handle.clone()).is_none(), Error::<T>::HandleIsNotUnique);

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

impl<T: Trait> Space<T> {

    pub fn create(
        id: SpaceId,
        created_by: T::AccountId,
        ipfs_hash: Vec<u8>,
        handle: Option<Vec<u8>>
    ) -> Self {
        Space {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            hidden: false,
            owner: created_by,
            handle,
            ipfs_hash,
            posts_count: 0,
            followers_count: 0,
            edit_history: Vec::new(),
            score: 0,
            permissions: None
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn ensure_space_owner(&self, who: T::AccountId) -> DispatchResult {
        ensure!(self.is_owner(&who), Error::<T>::NotASpaceOwner);
        Ok(())
    }

    pub fn increment_posts_count(&mut self) -> DispatchResult {
        self.posts_count = self.posts_count.checked_add(1).ok_or(Error::<T>::OverflowAddingPostOnSpace)?;
        Ok(())
    }

    pub fn ensure_space_stored(space_id: SpaceId) -> DispatchResult {
        ensure!(<SpaceById<T>>::exists(space_id), Error::<T>::SpaceNotFound);
        Ok(())
    }
}

impl<T: Trait> Post<T> {

    pub fn create(
        id: PostId,
        created_by: T::AccountId,
        space_id_opt: Option<SpaceId>,
        extension: PostExtension,
        ipfs_hash: Vec<u8>
    ) -> Self {
        Post {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            hidden: false,
            space_id: space_id_opt,
            extension,
            ipfs_hash,
            edit_history: Vec::new(),
            direct_replies_count: 0,
            total_replies_count: 0,
            shares_count: 0,
            upvotes_count: 0,
            downvotes_count: 0,
            score: 0
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.created.account == *account
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
            PostExtension::RegularPost | PostExtension::SharedPost(_) =>
                Ok(self.clone()),
            PostExtension::Comment(comment_ext) =>
                Module::post_by_id(comment_ext.root_post_id).ok_or_else(|| Error::<T>::PostNotFound.into()),
        }
    }

    pub fn get_space(&self) -> Result<Space<T>, DispatchError> {
        let root_post = self.get_root_post()?;

        let space_id = root_post.space_id.ok_or(Error::<T>::SpaceIdIsUndefined)?;
        let space = Module::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?;

        Ok(space)
    }

    pub fn ensure_post_stored(post_id: PostId) -> DispatchResult {
        ensure!(<PostById<T>>::exists(post_id), Error::<T>::PostNotFound);
        Ok(())
    }
}

impl<T: Trait> SpaceForRolesProvider for Module<T> {
    type AccountId = T::AccountId;
    type SpaceId = SpaceId;

    fn get_space(id: Self::SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError> {
        let space: Space<T> = Module::space_by_id(id).ok_or(Error::<T>::SpaceNotFound)?;

        Ok(SpaceForRoles {
            owner: space.owner,
            permissions: space.permissions
        })
    }

    fn is_space_follower(account: Self::AccountId, space_id: Self::SpaceId) -> bool {
        Module::<T>::space_followed_by_account((account, space_id))
    }
}
