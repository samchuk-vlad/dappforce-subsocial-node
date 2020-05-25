#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

pub mod functions;
// mod tests;

use sp_std::prelude::*;
use sp_std::collections::btree_set::BTreeSet;
use codec::{Encode, Decode};
use frame_support::{
  decl_module, decl_storage, decl_event, decl_error, ensure
};
use sp_runtime::RuntimeDebug;
use system::ensure_signed;

// TODO: import type from pallet-social after Blog will be renamed to Space
use pallet_utils::{WhoAndWhen, User, SpaceId};
use pallet_social::{Module as Social, Blog};


#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum SpacePermission {
  /// Create, update, grant and revoke roles in this space.
  ManageRoles,
  /// Create, update own and delete any subspaces in this space.
  ManageSubspaces,
  /// Create, update own and delete any root posts in this space.
  ManagePosts,
  /// Create, update own and delete any comments in this space.
  ManageComments,

  /// Act on behalf of this space within this space.
  RepresentSpaceInternally,
  /// Act on behalf of this space outside of this space.
  RepresentSpaceExternally,

  UpdateSpace,
  BlockUsers, // or BlockUsers

  // TODO what about 'DeleteSpace'? (too dangerous)

  // Related to subspaces in this space:
  CreateSubspaces,
  UpdateOwnSubspaces,
  DeleteOwnSubspaces,
  DeleteAnySubspaces,

  // Related to posts in this space:
  CreatePosts,
  UpdateOwnPosts,
  DeleteOwnPosts,
  DeleteAnyPosts,

  // Related to comments in this space:
  CreateComments,
  UpdateOwnComments,
  DeleteOwnComments,
  DeleteAnyComments,

  /// Upvote on any post or comment in this space.
  Upvote,
  /// Upvote on any post or comment in this space.
  Downvote,
  /// Share any post or comment from this space to another outer space.
  Share,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum PostPermission {
  // Related to comments on this post:
  CreateComments,
  UpdateOwnComments,
  DeleteOwnComments,

  // Related to this post and its comments:
  Upvote,
  Downvote,
  Share,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Role<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub id: RoleId,
  pub space_id: SpaceId,
  pub disabled: bool,
  pub ipfs_hash: Vec<u8>,
  pub permissions: BTreeSet<SpacePermission>
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct RoleUpdate {
  pub disabled: Option<bool>,
  pub ipfs_hash: Option<Vec<u8>>,
  pub permissions: Option<BTreeSet<SpacePermission>>,
}

type RoleId = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_timestamp::Trait + pallet_social::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Account has not permission to manage roles on this space
    NoPermissionToManageRoles,
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as PermissionsModule {
    /// Get role details by ids id.
    pub RoleById get(fn role_by_id): map RoleId => Option<Role<T>>;

    /// A list of all account ids and space ids that have this role.
    pub UsersByRoleId get(fn users_by_role_id): map RoleId => Vec<User<T::AccountId>>;

    /// A list of all role ids available in this space.
    pub RoleIdsBySpaceId get(fn role_ids_by_space_id): map SpaceId => Vec<RoleId>;

    /// A list of all role ids granted to this user (either account of space) within this space.
    pub InSpaceRoleIdsByUser get(fn in_space_role_ids_by_user): map (User<T::AccountId>, SpaceId) => Vec<RoleId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your pallet
    fn deposit_event() = default;

    /// Create a new role within this space with the list of particular permissions.
    /// `ipfs_hash` points to the off-chain content with such role info as name, description, color.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn create_role(origin, space_id: SpaceId, permissions: BTreeSet<SpacePermission>, ipfs_hash: Vec<u8>) {
      let who = ensure_signed(origin)?;

      let space: Blog<T> = Social::blog_by_id(space_id).ok_or("BlogNotFound")?;

      let is_user_has_permission = Self::is_user_has_permission(User::Account(who.clone()), SpacePermission::ManageRoles)?;
      ensure!(space.owner == who || is_user_has_permission, Error::<T>::NoPermissionToManageRoles);


    }

    /// Update an existing role on specified space.
    /// It is possible to either update permissions by overriding existing permissions,
    /// or update IPFS hash or both.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn update_role(origin, role_id: RoleId, update: RoleUpdate) {
      let who = ensure_signed(origin)?;
    }

    /// Delete the role from all associated storage items.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn delete_role(origin, role_id: RoleId, update: RoleUpdate) {
      let who = ensure_signed(origin)?;
    }

    /// Grant the role from the list of users.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn grant_role(origin, role_id: RoleId, users: Vec<User<T::AccountId>>) {
      let who = ensure_signed(origin)?;
    }

    /// Revoke the role from the list of users.
    /// Only the space owner, an user with `ManageRoles` permission or an user that has this role can execute this extrinsic.
    pub fn revoke_role(origin, role_id: RoleId, users: Vec<User<T::AccountId>>) {
      let who = ensure_signed(origin)?;
    }

    /// Disable the role. If the role is disabled, their permissions should not be taken into account.
    /// Should throw an error if the role is not enabled.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn disable_role(origin, role_id: RoleId) {
      let who = ensure_signed(origin)?;
    }

    /// Enable the role. Should throw an error if the role is not disabled.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn enable_role(origin, role_id: RoleId) {
      let who = ensure_signed(origin)?;
    }
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
   {
    EventTemplate(AccountId),
  }
);
