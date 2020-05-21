#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

pub mod functions;
mod tests;

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, traits::Get};
use sp_runtime::RuntimeDebug;
use system::ensure_signed;
use pallet_utils::WhoAndWhen;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Blog<T: Trait> {
  pub id: BlogId,
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub hidden: bool,

  // Can be updated by the owner:
  pub owner: T::AccountId,
  pub handle: Option<Vec<u8>>,
  pub ipfs_hash: Vec<u8>,

  pub posts_count: u16,
  pub followers_count: u32,

  pub edit_history: Vec<BlogHistoryRecord<T>>,

  pub score: i32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[allow(clippy::option_option)]
pub struct BlogUpdate {
  pub handle: Option<Option<Vec<u8>>>,
  pub ipfs_hash: Option<Vec<u8>>,
  pub hidden: Option<bool>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BlogHistoryRecord<T: Trait> {
  pub edited: WhoAndWhen<T>,
  pub old_data: BlogUpdate,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Post<T: Trait> {
  pub id: PostId,
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub hidden: bool,

  pub blog_id: Option<BlogId>,
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
  pub blog_id: Option<BlogId>,
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
pub struct CommentExt {
  parent_id: Option<PostId>,
  root_post_id: PostId,
}

impl Default for PostExtension {
  fn default() -> Self {
    PostExtension::RegularPost
  }
}

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

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SocialAccount<T: Trait> {
  pub followers_count: u32,
  pub following_accounts_count: u16,
  pub following_blogs_count: u16,
  pub reputation: u32,
  pub profile: Option<Profile<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Profile<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,

  pub username: Vec<u8>,
  pub ipfs_hash: Vec<u8>,

  pub edit_history: Vec<ProfileHistoryRecord<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileUpdate {
  pub username: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileHistoryRecord<T: Trait> {
  pub edited: WhoAndWhen<T>,
  pub old_data: ProfileUpdate,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum ScoringAction {
  UpvotePost,
  DownvotePost,
  SharePost,
  CreateComment,
  UpvoteComment,
  DownvoteComment,
  ShareComment,
  FollowBlog,
  FollowAccount,
}

impl Default for ScoringAction {
  fn default() -> Self {
    ScoringAction::FollowAccount
  }
}

pub type BlogId = u64;
pub type PostId = u64;
pub type ReactionId = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_timestamp::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

  /// The length in bytes of IPFS hash
  type IpfsHashLen: Get<u32>;

  /// Minimal length of blog handle
  type MinHandleLen: Get<u32>;

  /// Maximal length of blog handle
  type MaxHandleLen: Get<u32>;

  /// Minimal length of profile username
  type MinUsernameLen: Get<u32>;

  /// Maximal length of profile username
  type MaxUsernameLen: Get<u32>;

  /// Weights of the related social account actions
  type FollowBlogActionWeight: Get<i16>;
  type FollowAccountActionWeight: Get<i16>;
  type UpvotePostActionWeight: Get<i16>;
  type DownvotePostActionWeight: Get<i16>;
  type SharePostActionWeight: Get<i16>;
  type CreateCommentActionWeight: Get<i16>;
  type UpvoteCommentActionWeight: Get<i16>;
  type DownvoteCommentActionWeight: Get<i16>;
  type ShareCommentActionWeight: Get<i16>;

  /// Max comments depth
  type MaxCommentDepth: Get<u32>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Blog was not found by id
    BlogNotFound,
    /// Blog handle is too short
    HandleIsTooShort,
    /// Blog handle is too long
    HandleIsTooLong,
    /// Blog handle is not unique
    HandleIsNotUnique,
    /// Blog handle contains invalid characters
    HandleContainsInvalidChars,
    /// Nothing to update in blog
    NoUpdatesInBlog,
    /// Only blog owner can manage their blog
    NotABlogOwner,
    /// The current blog owner cannot transfer ownership to himself
    CannotTranferToCurrentOwner,
    /// There is no transfer ownership by blog that is provided
    NoPendingTransferOnBlog,
    /// The account is not allowed to apply transfer ownership
    NotAllowedToApplyOwnershipTransfer,
    /// The account is not allowed to reject transfer ownership
    NotAllowedToRejectOwnershipTransfer,

    /// Post was not found by id
    PostNotFound,
    /// Nothing to update in post
    NoUpdatesInPost,
    /// Only post author can manage their blog
    NotAnAuthor,
    /// Overflow caused adding post on blog
    OverflowAddingPostOnBlog,
    /// Cannot create post not defining blog_id
    BlogIdIsUndefined,
    /// Not allowed to create post/comment when entity is hidden
    BannedToCreateWhenHidden,
    /// Not allowed to follow blog when it's hidden
    BannedToFollowWhenHidden,
    /// Not allowed to create/update reaction to post/comment when entity is hidden
    BannedToChangeReactionWhenHidden,

    /// Unknown parent comment id
    UnknownParentComment,
    /// Post by parent_id is not of Comment extension
    NotACommentByParentId,
    /// New comment IPFS-hash is the same as old one
    CommentIPFSHashNotDiffer,
    /// Overflow adding comment on post
    OverflowAddingCommentOnPost,
    /// Cannot update blog id on Comment
    CannotUpdateBlogIdOnComment,
    /// Max comment depth reached
    MaxCommentDepthReached,

    /// Reaction was not found by id
    ReactionNotFound,
    /// Account has already reacted to this post/comment
    AccountAlreadyReacted,
    /// Account has not reacted to this post/comment yet
    AccountNotYetReacted,
    /// There is no post/comment reaction by account that could be deleted
    ReactionByAccountNotFound,
    /// Overflow caused upvoting post/comment
    OverflowUpvoting,
    /// Overflow caused downvoting post/comment
    OverflowDownvoting,
    /// Only reaction owner can update their reaction
    NotAReactionOwner,
    /// New reaction kind is the same as old one
    NewReactionKindNotDiffer,

    /// Account is already following this blog
    AccountIsFollowingBlog,
    /// Account is not following this blog
    AccountIsNotFollowingBlog,
    /// Account can not follow itself
    AccountCannotFollowItself,
    /// Account can not unfollow itself
    AccountCannotUnfollowItself,
    /// Account is already followed
    AccountIsAlreadyFollowed,
    /// Account is not followed by current follower
    AccountIsNotFollowed,
    /// Underflow unfollowing blog
    UnderflowUnfollowingBlog,
    /// Overflow caused following blog
    OverflowFollowingBlog,
    /// Overflow caused following account
    OverflowFollowingAccount,
    /// Underflow caused unfollowing account
    UnderflowUnfollowingAccount,

    /// Social account was not found by id
    SocialAccountNotFound,
    /// Follower social account was not found by id
    FollowerAccountNotFound,
    /// Social account that is being followed was not found by id
    FollowedAccountNotFound,

    /// IPFS-hash is not correct
    IpfsIsIncorrect,

    /// Out of bounds updating blog score
    OutOfBoundsUpdatingBlogScore,
    /// Out of bounds reverting blog score
    OutOfBoundsRevertingBlogScore,
    /// Out of bounds updating post score
    OutOfBoundsUpdatingPostScore,
    /// Out of bounds reverting post score
    OutOfBoundsRevertingPostScore,
    /// Post extension is a comment
    PostIsAComment,
    /// Out of bounds updating comment score
    OutOfBoundsUpdatingCommentScore,
    /// Out of bounds reverting comment score
    OutOfBoundsRevertingCommentScore,
    /// Post extension is not a comment
    PostIsNotAComment,
    /// Out of bounds updating social account reputation
    OutOfBoundsUpdatingAccountReputation,
    /// Scored account reputation difference by account and action not found
    ReputationDiffNotFound,

    /// Original post not found when sharing
    OriginalPostNotFound,
    /// Overflow caused on total shares counter when sharing post/comment
    OverflowTotalShares,
    /// Overflow caused on shares by account counter when sharing post/comment
    OverflowPostShares,
    /// Cannot share post that is not a RegularPost
    CannotShareSharedPost,

    /// Profile for this account already exists
    ProfileAlreadyExists,
    /// Nothing to update in a profile
    NoUpdatesInProfile,
    /// Account has no profile yet
    ProfileDoesNotExist,
    /// Profile username is busy
    UsernameIsBusy,
    /// Username is too short
    UsernameIsTooShort,
    /// Username is too long
    UsernameIsTooLong,
    /// Username is not alphanumeric
    UsernameIsNotAlphanumeric,
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as TemplateModule {
    pub BlogById get(blog_by_id): map BlogId => Option<Blog<T>>;
    pub PostById get(post_by_id): map PostId => Option<Post<T>>;
    pub ReactionById get(reaction_by_id): map ReactionId => Option<Reaction<T>>;
    pub SocialAccountById get(social_account_by_id): map T::AccountId => Option<SocialAccount<T>>;

    pub BlogIdsByOwner get(blog_ids_by_owner): map T::AccountId => Vec<BlogId>;
    pub PostIdsByBlogId get(post_ids_by_blog_id): map BlogId => Vec<PostId>;

    pub ReplyIdsByPostId get(reply_ids_by_post_id): map PostId => Vec<PostId>;

    pub ReactionIdsByPostId get(reaction_ids_by_post_id): map PostId => Vec<ReactionId>;
    pub PostReactionIdByAccount get(post_reaction_id_by_account): map (T::AccountId, PostId) => ReactionId;

    pub BlogIdByHandle get(blog_id_by_handle): map Vec<u8> => Option<BlogId>;

    pub BlogsFollowedByAccount get(blogs_followed_by_account): map T::AccountId => Vec<BlogId>;
    pub BlogFollowers get(blog_followers): map BlogId => Vec<T::AccountId>;
    pub BlogFollowedByAccount get(blog_followed_by_account): map (T::AccountId, BlogId) => bool;

    pub AccountFollowedByAccount get(account_followed_by_account): map (T::AccountId, T::AccountId) => bool;
    pub AccountsFollowedByAccount get(accounts_followed_by_account): map T::AccountId => Vec<T::AccountId>;
    pub AccountFollowers get(account_followers): map T::AccountId => Vec<T::AccountId>;

    pub NextBlogId get(next_blog_id): BlogId = 1;
    pub NextPostId get(next_post_id): PostId = 1;
    pub NextReactionId get(next_reaction_id): ReactionId = 1;

    pub AccountReputationDiffByAccount get(account_reputation_diff_by_account): map (T::AccountId, T::AccountId, ScoringAction) => Option<i16>; // TODO shorten name (?refactor)
    pub PostScoreByAccount get(post_score_by_account): map (T::AccountId, PostId, ScoringAction) => Option<i16>;

    pub PostSharesByAccount get(post_shares_by_account): map (T::AccountId, PostId) => u16;
    pub SharedPostIdsByOriginalPostId get(shared_post_ids_by_original_post_id): map PostId => Vec<PostId>;

    pub AccountByProfileUsername get(account_by_profile_username): map Vec<u8> => Option<T::AccountId>;

    pub PendingBlogOwner get(pending_blog_owner): map BlogId => Option<T::AccountId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    /// The length in bytes of IPFS hash
    const IpfsHashLen: u32 = T::IpfsHashLen::get();

    /// Minimal length of blog handle
    const MinHandleLen: u32 = T::MinHandleLen::get();

    /// Maximal length of blog handle
    const MaxHandleLen: u32 = T::MaxHandleLen::get();

    /// Minimal length of profile username
    const MinUsernameLen: u32 = T::MinUsernameLen::get();

    /// Maximal length of profile username
    const MaxUsernameLen: u32 = T::MaxUsernameLen::get();

    /// Weights of the related social account actions
    const FollowBlogActionWeight: i16 = T::FollowBlogActionWeight::get();
    const FollowAccountActionWeight: i16 = T::FollowAccountActionWeight::get();
    const UpvotePostActionWeight: i16 = T::UpvotePostActionWeight::get();
    const DownvotePostActionWeight: i16 = T::DownvotePostActionWeight::get();
    const SharePostActionWeight: i16 = T::SharePostActionWeight::get();
    const CreateCommentActionWeight: i16 = T::CreateCommentActionWeight::get();
    const UpvoteCommentActionWeight: i16 = T::UpvoteCommentActionWeight::get();
    const DownvoteCommentActionWeight: i16 = T::DownvoteCommentActionWeight::get();
    const ShareCommentActionWeight: i16 = T::ShareCommentActionWeight::get();

    /// Max comments depth
    const MaxCommentDepth: u32 = T::MaxCommentDepth::get();

    // Initializing events
    // this is needed only if you are using events in your pallet
    fn deposit_event() = default;

    pub fn create_blog(origin, handle_opt: Option<Vec<u8>>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      Self::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let mut handle: Vec<u8> = Vec::new();
      if let Some(original_handle) = handle_opt.clone() {
        handle = Self::lowercase_and_validate_a_handle(original_handle)?;
      }

      let blog_id = Self::next_blog_id();
      let new_blog: &mut Blog<T> = &mut Blog {
        id: blog_id,
        created: WhoAndWhen::<T>::new(owner.clone()),
        updated: None,
        hidden: false,
        owner: owner.clone(),
        handle: handle_opt,
        ipfs_hash,
        posts_count: 0,
        followers_count: 0,
        edit_history: Vec::new(),
        score: 0
      };

      // Blog creator automatically follows their blog:
      Self::add_blog_follower(owner.clone(), new_blog)?;

      if !handle.is_empty() {
        BlogIdByHandle::insert(handle, blog_id);
      }

      <BlogById<T>>::insert(blog_id, new_blog);
      <BlogIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(blog_id));
      NextBlogId::mutate(|n| { *n += 1; });
      Self::deposit_event(RawEvent::BlogCreated(owner, blog_id));
    }

    pub fn update_blog(origin, blog_id: BlogId, update: BlogUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.handle.is_some() ||
        update.ipfs_hash.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInBlog);

      let mut blog = Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?;

      blog.ensure_blog_owner(owner.clone())?;

      let mut fields_updated = 0;
      let mut new_history_record = BlogHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: BlogUpdate {handle: None, ipfs_hash: None, hidden: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != blog.ipfs_hash {
          Self::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(blog.ipfs_hash);
          blog.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != blog.hidden {
          new_history_record.old_data.hidden = Some(blog.hidden);
          blog.hidden = hidden;
          fields_updated += 1;
        }
      }

      if let Some(handle_opt) = update.handle {
        if handle_opt != blog.handle {
          if let Some(mut handle) = handle_opt.clone() {
            handle = Self::lowercase_and_validate_a_handle(handle)?;
            BlogIdByHandle::insert(handle, blog_id);
          }
          if let Some(blog_handle) = blog.handle.clone() {
            BlogIdByHandle::remove(blog_handle);
          }
          new_history_record.old_data.handle = Some(blog.handle);
          blog.handle = handle_opt;
          fields_updated += 1;
        }
      }

      // Update this blog only if at least one field should be updated:
      if fields_updated > 0 {
        blog.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        blog.edit_history.push(new_history_record);
        <BlogById<T>>::insert(blog_id, blog);
        Self::deposit_event(RawEvent::BlogUpdated(owner, blog_id));
      }
    }

    pub fn transfer_blog_ownership(origin, blog_id: BlogId, transfer_to: T::AccountId) {
      let who = ensure_signed(origin)?;

      let blog = Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?;
      blog.ensure_blog_owner(who.clone())?;

      ensure!(who != transfer_to, Error::<T>::CannotTranferToCurrentOwner);
      Blog::<T>::ensure_blog_stored(blog_id)?;

      <PendingBlogOwner<T>>::insert(blog_id, transfer_to);
      Self::deposit_event(RawEvent::BlogOwnershipTransferCreated(who, blog_id));
    }

    pub fn accept_pending_ownership(origin, blog_id: BlogId) {
      let who = ensure_signed(origin)?;

      let transfer_to = Self::pending_blog_owner(blog_id).ok_or(Error::<T>::NoPendingTransferOnBlog)?;
      ensure!(who == transfer_to, Error::<T>::NotAllowedToApplyOwnershipTransfer);

      if let Some(mut blog) = Self::blog_by_id(blog_id) {
        blog.owner = who.clone();
        <BlogById<T>>::insert(blog_id, blog);
      } else {
        <PendingBlogOwner<T>>::remove(blog_id);
        return Err(Error::<T>::BlogNotFound.into())
      }

      <PendingBlogOwner<T>>::remove(blog_id);
      Self::deposit_event(RawEvent::BlogOwnerChanged(who, blog_id));
    }

    pub fn reject_pending_ownership(origin, blog_id: BlogId) {
      let who = ensure_signed(origin)?;

      let blog = Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?;
      let transfer_to = Self::pending_blog_owner(blog_id).ok_or(Error::<T>::NoPendingTransferOnBlog)?;
      ensure!(who == transfer_to || who == blog.owner, Error::<T>::NotAllowedToRejectOwnershipTransfer);

      <PendingBlogOwner<T>>::remove(blog_id);
      Self::deposit_event(RawEvent::BlogOwnershipTransferCreated(who, blog_id));
    }

    pub fn follow_blog(origin, blog_id: BlogId) {
      let follower = ensure_signed(origin)?;

      let blog = &mut (Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?);
      ensure!(!Self::blog_followed_by_account((follower.clone(), blog_id)), Error::<T>::AccountIsFollowingBlog);
      ensure!(!blog.hidden, Error::<T>::BannedToFollowWhenHidden);

      Self::add_blog_follower(follower, blog)?;
      <BlogById<T>>::insert(blog_id, blog);
    }

    pub fn unfollow_blog(origin, blog_id: BlogId) {
      let follower = ensure_signed(origin)?;

      let blog = &mut (Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?);
      ensure!(Self::blog_followed_by_account((follower.clone(), blog_id)), Error::<T>::AccountIsNotFollowingBlog);

      let mut social_account = Self::social_account_by_id(follower.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
      social_account.following_blogs_count = social_account.following_blogs_count
        .checked_sub(1)
        .ok_or(Error::<T>::UnderflowUnfollowingBlog)?;
      blog.followers_count = blog.followers_count.checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingBlog)?;

      if blog.created.account != follower {
        let author = blog.created.account.clone();
        if let Some(score_diff) = Self::account_reputation_diff_by_account((follower.clone(), author.clone(), ScoringAction::FollowBlog)) {
          blog.score = blog.score.checked_sub(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingBlogScore)?;
          Self::change_social_account_reputation(author, follower.clone(), -score_diff, ScoringAction::FollowBlog)?;
        }
      }

      <BlogsFollowedByAccount<T>>::mutate(follower.clone(), |blog_ids| Self::vec_remove_on(blog_ids, blog_id));
      <BlogFollowers<T>>::mutate(blog_id, |account_ids| Self::vec_remove_on(account_ids, follower.clone()));
      <BlogFollowedByAccount<T>>::remove((follower.clone(), blog_id));
      <SocialAccountById<T>>::insert(follower.clone(), social_account);
      <BlogById<T>>::insert(blog_id, blog);

      Self::deposit_event(RawEvent::BlogUnfollowed(follower, blog_id));
    }

    pub fn follow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotFollowItself);
      ensure!(!<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), Error::<T>::AccountIsAlreadyFollowed);

      let mut follower_account = Self::get_or_new_social_account(follower.clone());
      let mut followed_account = Self::get_or_new_social_account(account.clone());

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_add(1).ok_or(Error::<T>::OverflowFollowingAccount)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_add(1).ok_or(Error::<T>::OverflowFollowingAccount)?;

      Self::change_social_account_reputation(account.clone(), follower.clone(),
        Self::get_score_diff(follower_account.reputation, ScoringAction::FollowAccount),
        ScoringAction::FollowAccount
      )?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |ids| ids.push(follower.clone()));
      <AccountFollowedByAccount<T>>::insert((follower.clone(), account.clone()), true);

      Self::deposit_event(RawEvent::AccountFollowed(follower, account));
    }

    pub fn unfollow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotUnfollowItself);

      let mut follower_account = Self::social_account_by_id(follower.clone()).ok_or(Error::<T>::FollowerAccountNotFound)?;
      let mut followed_account = Self::social_account_by_id(account.clone()).ok_or(Error::<T>::FollowedAccountNotFound)?;

      ensure!(<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), Error::<T>::AccountIsNotFollowed);

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingAccount)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingAccount)?;

      let reputation_diff = Self::account_reputation_diff_by_account(
        (follower.clone(), account.clone(), ScoringAction::FollowAccount)
      ).ok_or(Error::<T>::ReputationDiffNotFound)?;
      Self::change_social_account_reputation(account.clone(), follower.clone(),
        reputation_diff,
        ScoringAction::FollowAccount
      )?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |account_ids| Self::vec_remove_on(account_ids, account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |account_ids| Self::vec_remove_on(account_ids, follower.clone()));
      <AccountFollowedByAccount<T>>::remove((follower.clone(), account.clone()));

      Self::deposit_event(RawEvent::AccountUnfollowed(follower, account));
    }

    pub fn create_profile(origin, username: Vec<u8>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      let mut social_account = Self::get_or_new_social_account(owner.clone());
      ensure!(social_account.profile.is_none(), Error::<T>::ProfileAlreadyExists);
      Self::is_username_valid(username.clone())?;
      Self::is_ipfs_hash_valid(ipfs_hash.clone())?;

      social_account.profile = Some(
        Profile {
          created: WhoAndWhen::<T>::new(owner.clone()),
          updated: None,
          username: username.clone(),
          ipfs_hash,
          edit_history: Vec::new()
        }
      );
      <AccountByProfileUsername<T>>::insert(username, owner.clone());
      <SocialAccountById<T>>::insert(owner.clone(), social_account);

      Self::deposit_event(RawEvent::ProfileCreated(owner));
    }

    pub fn update_profile(origin, update: ProfileUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.username.is_some() ||
        update.ipfs_hash.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInProfile);

      let mut social_account = Self::social_account_by_id(owner.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
      let mut profile = social_account.profile.ok_or(Error::<T>::ProfileDoesNotExist)?;
      let mut is_update_applied = false;
      let mut new_history_record = ProfileHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: ProfileUpdate {username: None, ipfs_hash: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != profile.ipfs_hash {
          Self::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(profile.ipfs_hash);
          profile.ipfs_hash = ipfs_hash;
          is_update_applied = true;
        }
      }

      if let Some(username) = update.username {
        if username != profile.username {
          Self::is_username_valid(username.clone())?;
          <AccountByProfileUsername<T>>::remove(profile.username.clone());
          <AccountByProfileUsername<T>>::insert(username.clone(), owner.clone());
          new_history_record.old_data.username = Some(profile.username);
          profile.username = username;
          is_update_applied = true;
        }
      }

      if is_update_applied {
        profile.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        profile.edit_history.push(new_history_record);
        social_account.profile = Some(profile);
        <SocialAccountById<T>>::insert(owner.clone(), social_account);

        Self::deposit_event(RawEvent::ProfileUpdated(owner));
      }
    }

    pub fn create_post(origin, blog_id_opt: Option<BlogId>, extension: PostExtension, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      Self::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let new_post_id = Self::next_post_id();
      let new_post: Post<T> = Post::create(new_post_id, owner.clone(), blog_id_opt, extension, ipfs_hash);

      // Get blog from either from blog_id_opt or extension if a Comment provided
      let mut blog = new_post.get_blog()?;
      ensure!(!blog.hidden, Error::<T>::BannedToCreateWhenHidden);

      let root_post = &mut (new_post.get_root_post()?);
      ensure!(!root_post.hidden, Error::<T>::BannedToCreateWhenHidden);

      match extension {
        PostExtension::RegularPost => {
          blog.ensure_blog_owner(owner.clone())?;
          blog.increment_posts_count()?;
        },
        PostExtension::Comment(comment_ext) => {
          root_post.total_replies_count = root_post.total_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;

          if let Some(parent_id) = comment_ext.parent_id {
            let mut parent_comment = Self::post_by_id(parent_id).ok_or(Error::<T>::UnknownParentComment)?;
            ensure!(parent_comment.is_comment(), Error::<T>::NotACommentByParentId);
            parent_comment.direct_replies_count = parent_comment.direct_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;

            let mut ancestors = Self::get_ancestors(parent_id);
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

          Self::change_post_score_by_extension(owner.clone(), root_post, ScoringAction::CreateComment)?;

          <PostById<T>>::insert(comment_ext.root_post_id, root_post);
        },
        PostExtension::SharedPost(post_id) => {
          let post = &mut (Self::post_by_id(post_id).ok_or(Error::<T>::OriginalPostNotFound)?);
          ensure!(!post.is_shared_post(), Error::<T>::CannotShareSharedPost);
          blog.posts_count = blog.posts_count.checked_add(1).ok_or(Error::<T>::OverflowAddingPostOnBlog)?;
          Self::share_post(owner.clone(), post, new_post_id)?;
        },
      }

      if !new_post.is_comment() {
        <BlogById<T>>::insert(blog.id, blog.clone());
        PostIdsByBlogId::mutate(blog.id, |ids| ids.push(new_post_id));
      }

      <PostById<T>>::insert(new_post_id, new_post);
      NextPostId::mutate(|n| { *n += 1; });

      Self::deposit_event(RawEvent::PostCreated(owner, new_post_id));
    }

    pub fn update_post(origin, post_id: PostId, update: PostUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.blog_id.is_some() ||
        update.ipfs_hash.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInPost);

      let mut post = Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?;

      ensure!(owner == post.created.account, Error::<T>::NotAnAuthor);

      let mut fields_updated = 0;
      let mut new_history_record = PostHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: PostUpdate {blog_id: None, ipfs_hash: None, hidden: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != post.ipfs_hash {
          Self::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(post.ipfs_hash);
          post.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        } else if post.is_comment() {
          return Err(Error::<T>::CommentIPFSHashNotDiffer.into());
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != post.hidden {
          new_history_record.old_data.hidden = Some(post.hidden);
          post.hidden = hidden;
          fields_updated += 1;
        }
      }

      // Move this post to another blog:
      if let Some(blog_id) = update.blog_id {
        ensure!(!post.is_comment(), Error::<T>::CannotUpdateBlogIdOnComment);

        if let Some(post_blog_id) = post.blog_id {
          if blog_id != post_blog_id {
            Blog::<T>::ensure_blog_stored(blog_id)?;

            // Remove post_id from its old blog:
            PostIdsByBlogId::mutate(post_blog_id, |post_ids| Self::vec_remove_on(post_ids, post_id));

            // Add post_id to its new blog:
            PostIdsByBlogId::mutate(blog_id, |ids| ids.push(post_id));
            new_history_record.old_data.blog_id = post.blog_id;
            post.blog_id = Some(blog_id);
            fields_updated += 1;
          }
        }
      }

      // Update this post only if at least one field should be updated:
      if fields_updated > 0 {
        post.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        post.edit_history.push(new_history_record);
        <PostById<T>>::insert(post_id, post);

        Self::deposit_event(RawEvent::PostUpdated(owner, post_id));
      }
    }

    pub fn create_post_reaction(origin, post_id: PostId, kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      let post = &mut (Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?);
      ensure!(
        !<PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::AccountAlreadyReacted
      );

      let blog = post.get_blog()?;
      ensure!(!blog.hidden && !Self::is_root_post_hidden(post_id)?, Error::<T>::BannedToChangeReactionWhenHidden);

      let reaction_id = Self::new_reaction(owner.clone(), kind);

      match kind {
        ReactionKind::Upvote => post.upvotes_count = post.upvotes_count.checked_add(1).ok_or(Error::<T>::OverflowUpvoting)?,
        ReactionKind::Downvote => post.downvotes_count = post.downvotes_count.checked_add(1).ok_or(Error::<T>::OverflowDownvoting)?,
      }
      let action = Self::scoring_action_by_post_extension(post.extension, kind, false);

      if post.created.account != owner {
        Self::change_post_score_by_extension(owner.clone(), post, action)?;
      }
      else {
        <PostById<T>>::insert(post_id, post.clone());
      }

      ReactionIdsByPostId::mutate(post_id, |ids| ids.push(reaction_id));
      <PostReactionIdByAccount<T>>::insert((owner.clone(), post_id), reaction_id);
      Self::deposit_event(RawEvent::PostReactionCreated(owner, post_id, reaction_id));
    }

    pub fn update_post_reaction(origin, post_id: PostId, reaction_id: ReactionId, new_kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::AccountNotYetReacted
      );

      let mut reaction = Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?;
      let post = &mut (Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?);

      let blog = post.get_blog()?;
      ensure!(!blog.hidden && !post.hidden, Error::<T>::BannedToChangeReactionWhenHidden);

      ensure!(owner == reaction.created.account, Error::<T>::NotAReactionOwner);
      ensure!(reaction.kind != new_kind, Error::<T>::NewReactionKindNotDiffer);

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
      let action = Self::scoring_action_by_post_extension(post.extension, new_kind, false);
      let action_to_cancel = Self::scoring_action_by_post_extension(post.extension, new_kind, true);

      Self::change_post_score_by_extension(owner.clone(), post, action_to_cancel)?;
      Self::change_post_score_by_extension(owner.clone(), post, action)?;

      <ReactionById<T>>::insert(reaction_id, reaction);
      <PostById<T>>::insert(post_id, post);

      Self::deposit_event(RawEvent::PostReactionUpdated(owner, post_id, reaction_id));
    }

    pub fn delete_post_reaction(origin, post_id: PostId, reaction_id: ReactionId) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      let reaction = Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?;
      let post = &mut (Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?);

      ensure!(owner == reaction.created.account, Error::<T>::NotAReactionOwner);

      match reaction.kind {
        ReactionKind::Upvote => post.upvotes_count -= 1,
        ReactionKind::Downvote => post.downvotes_count -= 1,
      }
      let action_to_cancel = Self::scoring_action_by_post_extension(post.extension, reaction.kind, false);

      Self::change_post_score_by_extension(owner.clone(), post, action_to_cancel)?;

      <PostById<T>>::insert(post_id, post);
      <ReactionById<T>>::remove(reaction_id);
      ReactionIdsByPostId::mutate(post_id, |ids| Self::vec_remove_on(ids, reaction_id));
      <PostReactionIdByAccount<T>>::remove((owner.clone(), post_id));

      Self::deposit_event(RawEvent::PostReactionDeleted(owner, post_id, reaction_id));
    }
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
   {
    BlogCreated(AccountId, BlogId),
    BlogUpdated(AccountId, BlogId),
    BlogDeleted(AccountId, BlogId),
    BlogOwnerChanged(AccountId, BlogId),

    BlogOwnershipTransferCreated(AccountId, BlogId),
    BlogOwnershipTransferRejected(AccountId, BlogId),

    BlogFollowed(AccountId, BlogId),
    BlogUnfollowed(AccountId, BlogId),

    AccountReputationChanged(AccountId, ScoringAction, u32),

    AccountFollowed(AccountId, AccountId),
    AccountUnfollowed(AccountId, AccountId),

    PostCreated(AccountId, PostId),
    PostUpdated(AccountId, PostId),
    PostDeleted(AccountId, PostId),
    PostShared(AccountId, PostId),

    PostReactionCreated(AccountId, PostId, ReactionId),
    PostReactionUpdated(AccountId, PostId, ReactionId),
    PostReactionDeleted(AccountId, PostId, ReactionId),

    ProfileCreated(AccountId),
    ProfileUpdated(AccountId),
  }
);
