#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use system::ensure_signed;

use pallet_permissions::SpacePermission;
use pallet_posts::{Module as Posts, PostById, PostId};
use pallet_spaces::Module as Spaces;
use pallet_utils::{vec_remove_on, WhoAndWhen};

// mod tests;

pub type ReactionId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum ReactionKind {
    Upvote,
    Downvote,
}

impl Default for ReactionKind {
    fn default() -> Self {
        ReactionKind::Upvote
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Reaction<T: Trait> {
    pub id: ReactionId,
    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub kind: ReactionKind,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + pallet_utils::Trait
    + pallet_posts::Trait
    + pallet_spaces::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub NextReactionId get(fn next_reaction_id): ReactionId = 1;
        pub ReactionById get(fn reaction_by_id): map ReactionId => Option<Reaction<T>>;
        pub ReactionIdsByPostId get(fn reaction_ids_by_post_id): map PostId => Vec<ReactionId>;
        pub PostReactionIdByAccount get(fn post_reaction_id_by_account): map (T::AccountId, PostId) => ReactionId;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        PostReactionCreated(AccountId, PostId, ReactionId),
        PostReactionUpdated(AccountId, PostId, ReactionId),
        PostReactionDeleted(AccountId, PostId, ReactionId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Reaction was not found by id.
        ReactionNotFound,
        /// Account has already reacted to this post/comment.
        AccountAlreadyReacted,
        /// There is no reaction by account on this post/comment.
        ReactionByAccountNotFound,
        /// Only reaction owner can update their reaction.
        NotReactionOwner,
        /// New reaction kind is the same as old one on this post/comment.
        SameReaction,

        /// Overflow caused by upvoting a post/comment.
        UpvoteOverflow,
        /// Overflow caused by downvoting a post/comment.
        DownvoteOverflow,

        /// Not allowed to react on a post/comment in a hidden space.
        CannotReactWhenSpaceHidden,
        /// Not allowed to react on a post/comment if a root post is hidden.
        CannotReactWhenPostHidden,

        /// User has no permission to upvote posts/comments in this space.
        NoPermissionToUpvote,
        /// User has no permission to downvote posts/comments in this space.
        NoPermissionToDownvote,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing events
    fn deposit_event() = default;

    pub fn create_post_reaction(origin, post_id: PostId, kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      let post = &mut Posts::require_post(post_id)?;
      ensure!(
        !<PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::AccountAlreadyReacted
      );

      let space = post.get_space()?;
      ensure!(!space.hidden, Error::<T>::CannotReactWhenSpaceHidden);
      ensure!(Posts::<T>::is_root_post_visible(post_id)?, Error::<T>::CannotReactWhenPostHidden);

      let reaction_id = Self::insert_new_reaction(owner.clone(), kind);

      match kind {
        ReactionKind::Upvote => {
          Spaces::ensure_account_has_space_permission(
            owner.clone(),
            &post.get_space()?,
            SpacePermission::Upvote,
            Error::<T>::NoPermissionToUpvote.into()
          )?;
          post.upvotes_count = post.upvotes_count.checked_add(1).ok_or(Error::<T>::UpvoteOverflow)?;
        },
        ReactionKind::Downvote => {
          Spaces::ensure_account_has_space_permission(
            owner.clone(),
            &post.get_space()?,
            SpacePermission::Downvote,
            Error::<T>::NoPermissionToDownvote.into()
          )?;
          post.downvotes_count = post.downvotes_count.checked_add(1).ok_or(Error::<T>::DownvoteOverflow)?;
        }
      }

      if post.is_owner(&owner) {
        <PostById<T>>::insert(post_id, post.clone());
      } else {
        // TODO old change_post_score
        // let action = Self::scoring_action_by_post_extension(post.extension, kind, false);
        // Self::change_post_score(owner.clone(), post, action)?;
      }

      ReactionIdsByPostId::mutate(post_id, |ids| ids.push(reaction_id));
      <PostReactionIdByAccount<T>>::insert((owner.clone(), post_id), reaction_id);
      Self::deposit_event(RawEvent::PostReactionCreated(owner, post_id, reaction_id));

      // TODO new change_post_score
      // T::ReactionHandler::on_post_reaction_created(...);
    }

    pub fn update_post_reaction(origin, post_id: PostId, reaction_id: ReactionId, new_kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      let mut reaction = Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?;
      let post = &mut Posts::require_post(post_id)?;

      ensure!(owner == reaction.created.account, Error::<T>::NotReactionOwner);
      ensure!(reaction.kind != new_kind, Error::<T>::SameReaction);

      // TODO old change_post_score
      // let old_kind = reaction.kind;

      reaction.kind = new_kind;
      reaction.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

      match new_kind {
        ReactionKind::Upvote => {
          post.upvotes_count += 1;
          post.downvotes_count -= 1;
        },
        ReactionKind::Downvote => {
          post.downvotes_count += 1;
          post.upvotes_count -= 1;
        },
      }

      // TODO old change_post_score
      // let action_to_cancel = Self::scoring_action_by_post_extension(post.extension, old_kind, true);
      // Self::change_post_score(owner.clone(), post, action_to_cancel)?;
      //
      // let action = Self::scoring_action_by_post_extension(post.extension, new_kind, false);
      // Self::change_post_score(owner.clone(), post, action)?;

      <ReactionById<T>>::insert(reaction_id, reaction);
      <PostById<T>>::insert(post_id, post);

      Self::deposit_event(RawEvent::PostReactionUpdated(owner, post_id, reaction_id));

      // TODO old change_post_score
      // T::ReactionHandler::on_post_reaction_updated(...);
    }

    pub fn delete_post_reaction(origin, post_id: PostId, reaction_id: ReactionId) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      let reaction = Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?;
      let post = &mut Posts::require_post(post_id)?;

      ensure!(owner == reaction.created.account, Error::<T>::NotReactionOwner);

      match reaction.kind {
        ReactionKind::Upvote => post.upvotes_count -= 1,
        ReactionKind::Downvote => post.downvotes_count -= 1,
      }

      // TODO old change_post_score
      // let action_to_cancel = Self::scoring_action_by_post_extension(post.extension, reaction.kind, false);
      // Self::change_post_score(owner.clone(), post, action_to_cancel)?;

      <PostById<T>>::insert(post_id, post);
      <ReactionById<T>>::remove(reaction_id);
      ReactionIdsByPostId::mutate(post_id, |ids| vec_remove_on(ids, reaction_id));
      <PostReactionIdByAccount<T>>::remove((owner.clone(), post_id));

      Self::deposit_event(RawEvent::PostReactionDeleted(owner, post_id, reaction_id));

      // TODO new change_post_score
      // T::ReactionHandler::on_post_reaction_deleted(...);
    }
  }
}

impl<T: Trait> Module<T> {

    // FIXME: don't add reaction in storage before the checks in 'create_reaction' are done
    pub fn insert_new_reaction(account: T::AccountId, kind: ReactionKind) -> ReactionId {
        let id = Self::next_reaction_id();
        let reaction: Reaction<T> = Reaction {
            id,
            created: WhoAndWhen::<T>::new(account),
            updated: None,
            kind
        };

        <ReactionById<T>>::insert(id, reaction);
        NextReactionId::mutate(|n| { *n += 1; });

        id
    }
}
