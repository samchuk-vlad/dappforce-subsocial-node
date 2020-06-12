#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult}, ensure, traits::Get,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use system::ensure_signed;

use pallet_permissions::SpacePermission;
use pallet_spaces::{Module as Spaces, SpaceById};
use pallet_utils::{Module as Utils, SpaceId, vec_remove_on, WhoAndWhen};

pub mod functions;
// mod tests;

pub type PostId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Post<T: Trait> {
    pub id: PostId,
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub hidden: bool,

    pub space_id: Option<SpaceId>,
    pub extension: PostExtension,

    pub ipfs_hash: Vec<u8>,
    pub edit_history: Vec<PostHistoryRecord<T>>,

    pub direct_replies_count: u16,
    pub total_replies_count: u32,

    pub shares_count: u16,
    pub upvotes_count: u16,
    pub downvotes_count: u16,

    pub score: i32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostUpdate {
    pub space_id: Option<SpaceId>,
    pub ipfs_hash: Option<Vec<u8>>,
    pub hidden: Option<bool>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostHistoryRecord<T: Trait> {
    pub edited: WhoAndWhen<T>,
    pub old_data: PostUpdate,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum PostExtension {
    RegularPost,
    Comment(CommentExt),
    SharedPost(PostId),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
// TODO rename: CommentExt -> Comment
pub struct CommentExt {
    pub parent_id: Option<PostId>,
    pub root_post_id: PostId,
}

impl Default for PostExtension {
    fn default() -> Self {
        PostExtension::RegularPost
    }
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_spaces::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Max comments depth
    type MaxCommentDepth: Get<u32>;

    type OnBeforePostShared: OnBeforePostShared<Self>;
}

/// Handler that will be called right before post is shared.
pub trait OnBeforePostShared<T: Trait> {
    fn on_before_post_shared(account: T::AccountId, original_post: &mut Post<T>) -> DispatchResult;
}

impl<T: Trait> OnBeforePostShared<T> for () {
    fn on_before_post_shared(_account: T::AccountId, _original_post: &mut Post<T>) -> DispatchResult {
        Ok(())
    }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub NextPostId get(fn next_post_id): PostId = 1;
        pub PostById get(fn post_by_id): map PostId => Option<Post<T>>;
        pub ReplyIdsByPostId get(fn reply_ids_by_post_id): map PostId => Vec<PostId>;
        pub PostIdsBySpaceId get(fn post_ids_by_space_id): map SpaceId => Vec<PostId>;

        // TODO rename 'Shared...' to 'Sharing...'
        pub SharedPostIdsByOriginalPostId get(fn shared_post_ids_by_original_post_id): map PostId => Vec<PostId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        PostCreated(AccountId, PostId),
        PostUpdated(AccountId, PostId),
        PostDeleted(AccountId, PostId),
        PostShared(AccountId, PostId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {

        // Post related errors:

        /// Post was not found by id.
        PostNotFound,
        /// Nothing to update in post.
        NoUpdatesForPost,
        /// Overflow caused adding post to space.
        PostsCountOverflow,
        /// Cannot create post not defining space_id.
        SpaceIdIsUndefined,
        /// Not allowed to create a post/comment when a scope (space or root post) is hidden.
        CannotCreateInHiddenScope,

        // Sharing related errors:

        /// Original post not found when sharing.
        OriginalPostNotFound,
        /// Overflow caused by total shares counter when sharing post/comment.
        PostSharesOverflow,
        /// Cannot share a post that shares another post.
        CannotShareSharingPost,

        // Comment related errors:

        /// Unknown parent comment id.
        UnknownParentComment,
        /// Post by parent_id is not of Comment extension.
        NotACommentByParentId,
        /// Overflow adding comment on post.
        OverflowAddingCommentOnPost,
        /// Cannot update space id on comment.
        CannotUpdateSpaceIdOnComment,
        /// Max comment depth reached.
        MaxCommentDepthReached,
        /// Only comment author can update his comment.
        NotACommentAuthor,
        /// Post extension is not a comment.
        PostIsNotAComment,

        // Permissions related errors:

        /// User has no permission to create root posts in this space.
        NoPermissionToCreatePosts,
        /// User has no permission to create comments (aka replies) in this space.
        NoPermissionToCreateComments,
        /// User has no permission to share posts/comments from this space to another space.
        NoPermissionToShare,
        /// User is not a post author and has no permission to update posts in this space.
        NoPermissionToUpdateAnyPost,
        /// A post owner is not allowed to update their own posts in this space.
        NoPermissionToUpdateOwnPosts,
        /// A comment owner is not allowed to update their own comments in this space.
        NoPermissionToUpdateOwnComments,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    const MaxCommentDepth: u32 = T::MaxCommentDepth::get();

    // Initializing events
    fn deposit_event() = default;

    pub fn create_post(origin, space_id_opt: Option<SpaceId>, extension: PostExtension, ipfs_hash: Vec<u8>) {
      let creator = ensure_signed(origin)?;

      Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let new_post_id = Self::next_post_id();
      let new_post: Post<T> = Post::new(new_post_id, creator.clone(), space_id_opt, extension, ipfs_hash);

      // Get space from either from space_id_opt or extension if a Comment provided
      let mut space = new_post.get_space()?;
      ensure!(!space.hidden, Error::<T>::CannotCreateInHiddenScope);

      let root_post = &mut new_post.get_root_post()?;
      ensure!(!root_post.hidden, Error::<T>::CannotCreateInHiddenScope);

      // Check permissions
      match extension {
        PostExtension::RegularPost | PostExtension::SharedPost(_) => {
          Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &space,
            SpacePermission::CreatePosts,
            Error::<T>::NoPermissionToCreatePosts.into()
          )?;
        },
        PostExtension::Comment(_) => {
          Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &space,
            SpacePermission::CreateComments,
            Error::<T>::NoPermissionToCreateComments.into()
          )?;
        }
      }

      match extension {
        PostExtension::RegularPost => {
          space.increment_posts_count()?;
        },

        PostExtension::SharedPost(post_id) => {
          let original_post = &mut Self::post_by_id(post_id).ok_or(Error::<T>::OriginalPostNotFound)?;
          ensure!(!original_post.is_sharing_post(), Error::<T>::CannotShareSharingPost);

          // Check if it's allowed to share a post from the space of original post.
          Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &original_post.get_space()?,
            SpacePermission::Share,
            Error::<T>::NoPermissionToShare.into()
          )?;

          space.posts_count = space.posts_count.checked_add(1).ok_or(Error::<T>::PostsCountOverflow)?;
          Self::share_post(creator.clone(), original_post, new_post_id)?;
        },

        PostExtension::Comment(comment_ext) => {
          root_post.total_replies_count = root_post.total_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;

          if let Some(parent_id) = comment_ext.parent_id {
            let mut parent_comment = Self::post_by_id(parent_id).ok_or(Error::<T>::UnknownParentComment)?;
            ensure!(parent_comment.is_comment(), Error::<T>::NotACommentByParentId);
            parent_comment.direct_replies_count = parent_comment.direct_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;

            let mut ancestors = Self::get_post_ancestors(parent_id);
            ancestors[0] = parent_comment;
            ensure!(ancestors.len() < T::MaxCommentDepth::get() as usize, Error::<T>::MaxCommentDepthReached);
            for mut post in ancestors {
              post.total_replies_count = post.total_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;
              <PostById<T>>::insert(post.id, post.clone());
            }

            ReplyIdsByPostId::mutate(parent_id, |ids| ids.push(new_post_id));
          } else {
            root_post.direct_replies_count = root_post.direct_replies_count.checked_add(1)
              .ok_or(Error::<T>::OverflowAddingCommentOnPost)?;

            ReplyIdsByPostId::mutate(comment_ext.root_post_id, |ids| ids.push(new_post_id));
          }

          // TODO old change root post score on new comment
          // Self::change_post_score(creator.clone(), root_post, ScoringAction::CreateComment)?;

          // TODO new before_comment_created
          // T::BeforeCommentCreated::before_comment_created(...);

          PostById::insert(comment_ext.root_post_id, root_post);
        }
      }

      if !new_post.is_comment() {
        SpaceById::insert(space.id, space.clone());
        PostIdsBySpaceId::mutate(space.id, |ids| ids.push(new_post_id));
      }

      PostById::insert(new_post_id, new_post);
      NextPostId::mutate(|n| { *n += 1; });

      Self::deposit_event(RawEvent::PostCreated(creator, new_post_id));
    }

    pub fn update_post(origin, post_id: PostId, update: PostUpdate) {
      let editor = ensure_signed(origin)?;

      let has_updates =
        update.space_id.is_some() ||
        update.ipfs_hash.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForPost);

      let mut post = Self::require_post(post_id)?;

      let is_owner = post.is_owner(&editor);
      let is_comment = post.is_comment();

      let permission_to_check: SpacePermission;
      let permission_error: DispatchError;

      if is_comment {
        if is_owner {
          permission_to_check = SpacePermission::UpdateOwnComments;
          permission_error = Error::<T>::NoPermissionToUpdateOwnComments.into();
        } else {
          return Err(Error::<T>::NotACommentAuthor.into());
        }
      } else { // not a comment
        if is_owner {
          permission_to_check = SpacePermission::UpdateOwnPosts;
          permission_error = Error::<T>::NoPermissionToUpdateOwnPosts.into();
        } else {
          permission_to_check = SpacePermission::UpdateAnyPost;
          permission_error = Error::<T>::NoPermissionToUpdateAnyPost.into();
        }
      }

      Spaces::ensure_account_has_space_permission(
        editor.clone(),
        &post.get_space()?,
        permission_to_check,
        permission_error
      )?;

      let mut fields_updated = 0;
      let mut new_history_record = PostHistoryRecord {
        edited: WhoAndWhen::<T>::new(editor.clone()),
        old_data: PostUpdate {space_id: None, ipfs_hash: None, hidden: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != post.ipfs_hash {
          Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(post.ipfs_hash);
          post.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != post.hidden {
          new_history_record.old_data.hidden = Some(post.hidden);
          post.hidden = hidden;
          fields_updated += 1;
        }
      }

      // Move this post to another space:
      if let Some(space_id) = update.space_id {
        ensure!(!post.is_comment(), Error::<T>::CannotUpdateSpaceIdOnComment);

        if let Some(post_space_id) = post.space_id {
          if space_id != post_space_id {
            Spaces::<T>::ensure_space_exists(space_id)?;
            // TODO check that the current user has CreatePosts permission in new space_id.

            // Remove post_id from its old space:
            PostIdsBySpaceId::mutate(post_space_id, |post_ids| vec_remove_on(post_ids, post_id));

            // Add post_id to its new space:
            PostIdsBySpaceId::mutate(space_id, |ids| ids.push(post_id));
            new_history_record.old_data.space_id = post.space_id;
            post.space_id = Some(space_id);
            fields_updated += 1;
          }
        }
      }

      // Update this post only if at least one field should be updated:
      if fields_updated > 0 {
        post.updated = Some(WhoAndWhen::<T>::new(editor.clone()));
        post.edit_history.push(new_history_record);
        <PostById<T>>::insert(post_id, post);

        Self::deposit_event(RawEvent::PostUpdated(editor, post_id));
      }
    }
  }
}
