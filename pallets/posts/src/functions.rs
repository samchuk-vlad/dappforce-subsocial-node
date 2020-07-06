use frame_support::dispatch::DispatchResult;

use pallet_utils::SpaceId;

use super::*;

impl<T: Trait> Post<T> {

    pub fn new(
        id: PostId,
        created_by: T::AccountId,
        space_id_opt: Option<SpaceId>,
        extension: PostExtension,
        content: Content
    ) -> Self {
        Post {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            owner: created_by,
            extension,
            space_id: space_id_opt,
            content,
            hidden: false,
            direct_replies_count: 0,
            total_replies_count: 0,
            direct_hidden_replies_count: 0,
            total_hidden_replies_count: 0,
            shares_count: 0,
            upvotes_count: 0,
            downvotes_count: 0,
            score: 0
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn is_root_post(&self) -> bool {
        !self.is_comment()
    }

    pub fn is_comment(&self) -> bool {
        match self.extension {
            PostExtension::Comment(_) => true,
            _ => false,
        }
    }

    pub fn is_sharing_post(&self) -> bool {
        match self.extension {
            PostExtension::SharedPost(_) => true,
            _ => false,
        }
    }

    pub fn get_comment_ext(&self) -> Result<CommentExt, DispatchError> {
        match self.extension {
            PostExtension::Comment(comment_ext) => Ok(comment_ext),
            _ => Err(Error::<T>::NotComment.into())
        }
    }

    pub fn get_root_post(&self) -> Result<Post<T>, DispatchError> {
        match self.extension {
            PostExtension::RegularPost | PostExtension::SharedPost(_) =>
                Ok(self.clone()),
            PostExtension::Comment(comment) =>
                Module::require_post(comment.root_post_id),
        }
    }

    pub fn get_space(&self) -> Result<Space<T>, DispatchError> {
        let root_post = self.get_root_post()?;
        let space_id = root_post.space_id.ok_or(Error::<T>::PostHasNoSpaceId)?;
        Spaces::require_space(space_id)
    }

    pub fn try_get_space(&self) -> Option<Space<T>> {
        if self.is_comment() {
            return None
        }

        if let Some(space_id) = self.space_id {
            return Spaces::require_space(space_id).ok()
        }

        None
    }

    // TODO use macros to generate inc/dec fns for Space, Post.

    pub fn inc_direct_replies(&mut self) {
        self.direct_replies_count = self.direct_replies_count.saturating_add(1);
    }

    pub fn dec_direct_replies(&mut self) {
        self.direct_replies_count = self.direct_replies_count.saturating_sub(1);
    }

    pub fn inc_total_replies(&mut self) {
        self.total_replies_count = self.total_replies_count.saturating_add(1);
    }

    pub fn dec_total_replies(&mut self) {
        self.total_replies_count = self.total_replies_count.saturating_sub(1);
    }

    pub fn inc_direct_hidden_replies(&mut self) {
        self.direct_hidden_replies_count = self.direct_hidden_replies_count.saturating_add(1);
    }

    pub fn dec_direct_hidden_replies(&mut self) {
        self.direct_hidden_replies_count = self.direct_hidden_replies_count.saturating_sub(1);
    }

    pub fn inc_total_hidden_replies(&mut self) {
        self.total_hidden_replies_count = self.total_hidden_replies_count.saturating_add(1);
    }

    pub fn dec_total_hidden_replies(&mut self) {
        self.total_hidden_replies_count = self.total_hidden_replies_count.saturating_sub(1);
    }

    pub fn inc_shares(&mut self) {
        self.shares_count = self.shares_count.saturating_add(1);
    }

    pub fn dec_shares(&mut self) {
        self.shares_count = self.shares_count.saturating_sub(1);
    }

    pub fn inc_upvotes(&mut self) {
        self.upvotes_count = self.upvotes_count.saturating_add(1);
    }

    pub fn dec_upvotes(&mut self) {
        self.upvotes_count = self.upvotes_count.saturating_sub(1);
    }

    pub fn inc_downvotes(&mut self) {
        self.downvotes_count = self.downvotes_count.saturating_add(1);
    }

    pub fn dec_downvotes(&mut self) {
        self.downvotes_count = self.downvotes_count.saturating_sub(1);
    }

    #[allow(clippy::comparison_chain)]
    pub fn change_score(&mut self, diff: i16) {
        if diff > 0 {
            self.score = self.score.saturating_add(diff.abs() as i32);
        } else if diff < 0 {
            self.score = self.score.saturating_sub(diff.abs() as i32);
        }
    }
}

impl Default for PostUpdate {
    fn default() -> Self {
        PostUpdate {
            space_id: None,
            content: None,
            hidden: None
        }
    }
}

impl<T: Trait> Module<T> {

    /// Check that there is a `Post` with such `post_id` in the storage
    /// or return`PostNotFound` error.
    pub fn ensure_post_exists(post_id: PostId) -> DispatchResult {
        ensure!(<PostById<T>>::contains_key(post_id), Error::<T>::PostNotFound);
        Ok(())
    }

    /// Get `Post` by id from the storage or return `PostNotFound` error.
    pub fn require_post(post_id: SpaceId) -> Result<Post<T>, DispatchError> {
        Ok(Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?)
    }

    fn share_post(
        account: T::AccountId,
        original_post: &mut Post<T>,
        shared_post_id: PostId
    ) -> DispatchResult {
        original_post.inc_shares();

        T::PostScores::score_post_on_new_share(account.clone(), original_post)?;

        let original_post_id = original_post.id;
        PostById::insert(original_post_id, original_post.clone());
        SharedPostIdsByOriginalPostId::mutate(original_post_id, |ids| ids.push(shared_post_id));

        Self::deposit_event(RawEvent::PostShared(account, original_post_id));

        Ok(())
    }

    pub fn is_root_post_hidden(post_id: PostId) -> Result<bool, DispatchError> {
        let post = Self::require_post(post_id)?;
        let root_post = post.get_root_post()?;
        Ok(root_post.hidden)
    }

    pub fn is_root_post_visible(post_id: PostId) -> Result<bool, DispatchError> {
        Self::is_root_post_hidden(post_id).map(|v| !v)
    }

    pub fn mutate_post_by_id<F: FnOnce(&mut Post<T>)> (
        post_id: PostId,
        f: F
    ) -> Result<Post<T>, DispatchError> {
        <PostById<T>>::mutate(post_id, |post_opt| {
            if let Some(ref mut post) = post_opt.clone() {
                f(post);
                *post_opt = Some(post.clone());

                return Ok(post.clone());
            }

            Err(Error::<T>::PostNotFound.into())
        })
    }

    // TODO refactor to a tail recursion
    pub fn get_post_ancestors(post_id: PostId) -> Vec<Post<T>> {
        let mut ancestors: Vec<Post<T>> = Vec::new();

        if let Some(post) = Self::post_by_id(post_id) {
            ancestors.push(post.clone());
            if let Some(parent_id) = post.get_comment_ext().ok().unwrap().parent_id {
                ancestors.extend(Self::get_post_ancestors(parent_id).iter().cloned());
            }
        }

        ancestors
    }

    pub fn for_each_post_ancestor<F: FnMut(&mut Post<T>) + Copy> (
        post_id: PostId,
        f: F
    ) -> DispatchResult {
        let post = Self::mutate_post_by_id(post_id, f)?;

        if let PostExtension::Comment(comment_ext) = post.extension {
            if let Some(parent_id) = comment_ext.parent_id {
                Self::for_each_post_ancestor(parent_id, f)?;
            }
        }

        Ok(())
    }

    pub(crate) fn create_comment(
        creator: &T::AccountId,
        new_post_id: PostId,
        comment_ext: CommentExt,
        root_post: &mut Post<T>
    ) -> DispatchResult {
        let mut commented_post_id = root_post.id;

        root_post.inc_total_replies();

        if let Some(parent_id) = comment_ext.parent_id {
            let parent_comment = Self::post_by_id(parent_id).ok_or(Error::<T>::UnknownParentComment)?;
            ensure!(parent_comment.is_comment(), Error::<T>::NotACommentByParentId);

            let ancestors = Self::get_post_ancestors(parent_id);
            ensure!(ancestors.len() < T::MaxCommentDepth::get() as usize, Error::<T>::MaxCommentDepthReached);

            commented_post_id = parent_id;
        }

        T::PostScores::score_root_post_on_new_comment(creator, root_post)?;

        Self::for_each_post_ancestor(commented_post_id, |post| post.inc_total_replies())?;
        PostById::insert(root_post.id, root_post);
        Self::mutate_post_by_id(commented_post_id, |post| post.inc_direct_replies())?;
        ReplyIdsByPostId::mutate(commented_post_id, |ids| ids.push(new_post_id));

        Ok(())
    }

    pub(crate) fn create_sharing_post(
        creator: &T::AccountId,
        new_post_id: PostId,
        original_post_id: PostId,
        space: &mut Space<T>
    ) -> DispatchResult {
        let original_post = &mut Self::post_by_id(original_post_id)
            .ok_or(Error::<T>::OriginalPostNotFound)?;

        ensure!(!original_post.is_sharing_post(), Error::<T>::CannotShareSharingPost);

        // Check if it's allowed to share a post from the space of original post.
        Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &original_post.get_space()?,
            SpacePermission::Share,
            Error::<T>::NoPermissionToShare.into()
        )?;

        space.inc_posts();

        Self::share_post(creator.clone(), original_post, new_post_id)
    }
}
