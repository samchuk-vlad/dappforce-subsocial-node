#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_std::collections::{
  btree_map::BTreeMap
};
use codec::{Encode, Decode};
use frame_support::{
  decl_module,
  traits::Get
};
use sp_runtime::RuntimeDebug;

use self::PostPermission as PP;
use self::SpacePermission as SP;

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum SpacePermission {
  /// Create, update, grant and revoke roles in this space.
  ManageRoles,

  /// Act on behalf of this space within this space.
  RepresentSpaceInternally,
  /// Act on behalf of this space outside of this space.
  RepresentSpaceExternally,

  UpdateSpace,
  BlockUsers,

  // Related to subspaces in this space:
  CreateSubspaces,
  UpdateSubspaces,
  DeleteSubspaces,

  // Related to posts in this space:
  CreatePosts,
  UpdatePosts,
  DeletePosts,

  // Related to comments in this space:
  CreateComments,
  UpdateComments,
  DeleteComments,

  /// Upvote on any post or comment in this space.
  Upvote,
  /// Upvote on any post or comment in this space.
  Downvote,
  /// Share any post or comment from this space to another outer space.
  Share,
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum PostPermission {
  // Relate to this post:
  UpdatePost,
  DeletePost,

  // Related to comments on this post:
  CreateComments,
  UpdateComments,
  DeleteComments,

  // Related to this post and its comments:
  Upvote,
  Downvote,
  Share,
}

impl Into<SpacePermission> for PostPermission {
  fn into(self) -> SpacePermission {
    match self {
      PP::UpdatePost => SP::UpdatePosts,
      PP::DeletePost => SP::DeletePosts,

      PP::CreateComments => SP::CreateComments,
      PP::UpdateComments => SP::UpdateComments,
      PP::DeleteComments => SP::DeleteComments,

      PP::Upvote => SP::Upvote,
      PP::Downvote => SP::Downvote,
      PP::Share => SP::Share,
    }
  }
}

/*
Example of using built-in roles with permissions:

                  None    Owner    Follower    Everyone
-------------------------------------------------------
CreatePosts    |            X
-------------------------------------------------------
CreateComments |                      X
-------------------------------------------------------
Share          |                      X
-------------------------------------------------------
Upvote         |                                  X
-------------------------------------------------------
Downvote       |   X
-------------------------------------------------------

*/

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum BuiltinRole {

  /// None is allowed.
  None,

  /// An owner of this entity on which we are checking a permission.
  /// For example it could be an owner of a space or a post (comment).
  Owner,

  /// Owners and followers of this space allowed.
  Follower,

  /// Every user of this blockchain is allowed.
  Everyone,
}

pub type SpacePermissions = BTreeMap<SpacePermission, BuiltinRole>;

pub type PostPermissions = BTreeMap<PostPermission, BuiltinRole>;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
  type DefaultSpacePermissions: Get<SpacePermissions>;
  type DefaultPostPermissions: Get<PostPermissions>;
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    const DefaultSpacePermissions: SpacePermissions = T::DefaultSpacePermissions::get();
    const DefaultPostPermissions: PostPermissions = T::DefaultPostPermissions::get();
  }
}

impl<T: Trait> Module<T> {

  pub fn has_user_a_space_permission(
    is_owner: bool,
    is_follower: bool,
    space_perms: SpacePermissions,
    permission: SpacePermission,
  ) -> bool {

    // Try to find a permission in space overrides:
    let mut role_opt = space_perms.get(&permission);

    // Look into default space permissions,
    // if there is no permission override for this space:
    let default_perms = T::DefaultSpacePermissions::get();
    if role_opt.is_none() {
      role_opt = default_perms.get(&permission);
    }

    Self::is_user_in_role(
      is_owner,
      is_follower,
      role_opt
    )
  }

  pub fn has_user_a_post_permission(
    is_post_owner: bool,
    is_space_owner: bool,
    is_follower: bool,
    post_perms: PostPermissions,
    space_perms: SpacePermissions,
    permission: PostPermission,
  ) -> bool {

    // Try to find a permission in post/comment overrides:
    let mut role_opt = post_perms.get(&permission);

    // Look into default post permissions,
    // if there is no permission override for this post/comment:
    let default_perms = T::DefaultPostPermissions::get();
    if role_opt.is_none() {
      role_opt = default_perms.get(&permission);
    }

    if Self::is_user_in_role(
      is_post_owner,
      is_follower,
      role_opt
    ) {
      return true;
    }

    Self::has_user_a_space_permission(
      is_space_owner,
      is_follower,
      space_perms,
      permission.into()
    )
  }

  pub fn is_user_in_role(
    is_owner: bool,
    is_follower: bool,
    role_to_check: Option<&BuiltinRole>,
  ) -> bool {

    if let Some(role) = role_to_check {
      if *role == BuiltinRole::None {
        return false;
      } else if
        *role == BuiltinRole::Owner && is_owner ||
        *role == BuiltinRole::Follower && (is_owner || is_follower) ||
        *role == BuiltinRole::Everyone
      {
        return true;
      }
    }

    false
  }
}