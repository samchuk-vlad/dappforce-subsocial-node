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

use pallet_utils::SpaceId;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpacePermissionsContext {
  pub space_id: SpaceId,
  pub is_space_owner: bool,
  pub is_space_follower: bool,
  pub space_perms: SpacePermissions
}

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
  UpdateOwnSubspaces,
  UpdateAnySubspaces,
  DeleteOwnSubspaces,
  BlockSubspaces,

  // Related to posts in this space:
  CreatePosts,
  UpdateOwnPosts,
  UpdateAnyPosts,
  DeleteOwnPosts,
  BlockPosts,

  // Related to comments in this space:
  CreateComments,
  UpdateOwnComments,
  DeleteOwnComments,
  BlockComments,

  /// Upvote on any post or comment in this space.
  Upvote,
  /// Upvote on any post or comment in this space.
  Downvote,
  /// Share any post or comment from this space to another outer space.
  Share,
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum BuiltinRole {

  /// None is allowed.
  None,

  /// An owner of the Space on which we are checking a permission.
  SpaceOwner,

  /// Owners and followers of this space allowed.
  Follower,

  /// Every user of this blockchain is allowed.
  Everyone,
}

pub type SpacePermissions = BTreeMap<SpacePermission, BuiltinRole>;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
  type DefaultSpacePermissions: Get<SpacePermissions>;
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    const DefaultSpacePermissions: SpacePermissions = T::DefaultSpacePermissions::get();
  }
}

impl<T: Trait> Module<T> {

  pub fn has_user_a_space_permission(
    space_permissions_context: SpacePermissionsContext,
    permission: SpacePermission,
  ) -> Option<BuiltinRole> {

    // Try to find a permission in space overrides:
    let mut role_opt = space_permissions_context.space_perms.get(&permission);

    // Look into default space permissions,
    // if there is no permission override for this space:
    let default_perms = T::DefaultSpacePermissions::get();
    if role_opt.is_none() {
      role_opt = default_perms.get(&permission);
    }

    Self::is_user_in_role(
      &space_permissions_context,
      role_opt
    )
  }

  pub fn is_user_in_role(
    space_permissions_context: &SpacePermissionsContext,
    role_to_check: Option<&BuiltinRole>,
  ) -> Option<BuiltinRole> {

    let is_owner = space_permissions_context.is_space_owner;
    let is_follower = space_permissions_context.is_space_follower;

    if let Some(role) = role_to_check {
      if *role == BuiltinRole::None ||
        *role == BuiltinRole::SpaceOwner && is_owner ||
        *role == BuiltinRole::Follower && (is_owner || is_follower) ||
        *role == BuiltinRole::Everyone
      {
        return Some(role.clone());
      }
    }

    None
  }
}