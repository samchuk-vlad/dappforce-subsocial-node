#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{
  prelude::*,
  collections::btree_set::BTreeSet,
  iter::IntoIterator
};
use codec::{Encode, Decode};
use frame_support::{
  decl_module,
  traits::Get
};
use sp_runtime::RuntimeDebug;

use pallet_utils::SpaceId;

use self::PostPermission as PP;
use self::SpacePermission as SP;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpacePermissionsContext {
  pub space_id: SpaceId,
  pub is_space_owner: bool,
  pub is_space_follower: bool,
  pub space_perms: SpacePermissions
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostPermissionsContext {
  pub is_post_owner: bool,
  pub post_perms: PostPermissions
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

  OverridePostPermissions,
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum PostPermission {
  // Relate to this post:
  UpdateOwnPost,
  DeleteOwnPost,

  // Related to comments on this post:
  CreateComments,
  UpdateOwnComments,
  DeleteOwnComments,
  BlockComments,

  // Related to this post and its comments:
  Upvote,
  Downvote,
  Share,
}

impl Into<SpacePermission> for PostPermission {
  fn into(self) -> SpacePermission {
    match self {
      PP::UpdateOwnPost => SP::UpdateOwnPosts,
      PP::DeleteOwnPost => SP::DeleteOwnPosts,

      PP::CreateComments => SP::CreateComments,
      PP::UpdateOwnComments => SP::UpdateOwnComments,
      PP::DeleteOwnComments => SP::DeleteOwnComments,
      PP::BlockComments => SP::BlockComments,

      PP::Upvote => SP::Upvote,
      PP::Downvote => SP::Downvote,
      PP::Share => SP::Share,
    }
  }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpacePermissions {
  none: Option<SpacePermissionSet>,
  everyone: Option<SpacePermissionSet>,
  follower: Option<SpacePermissionSet>,
  space_owner: Option<SpacePermissionSet>,
  post_owner: Option<PostPermissionSet>
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostPermissions {
  none: Option<PostPermissionSet>,
  everyone: Option<PostPermissionSet>,
  follower: Option<PostPermissionSet>,
  space_owner: Option<PostPermissionSet>,
  post_owner: Option<PostPermissionSet>
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum BuiltinRole {

  /// None is allowed.
  None,

  /// An owner of the space on which we are checking a permission.
  SpaceOwner,

  /// Owners and followers of this space allowed.
  Follower,

  /// Every user of this blockchain is allowed.
  Everyone,

  /// An owner of the post on which we are checking a permission.
  PostOwner,
}

impl IntoIterator for SpacePermissions {
  type Item = SpacePermissionSet;
  type IntoIter = sp_std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    let mut post_owner_converted = SpacePermissionSet::new();
    if let Some(post_owner) = self.post_owner {
      for permission in post_owner.iter() {
        post_owner_converted.insert(permission.clone().into());
      }
    }

    vec![
      self.none.unwrap_or(SpacePermissionSet::new()),
      self.space_owner.unwrap_or(SpacePermissionSet::new()),
      self.follower.unwrap_or(SpacePermissionSet::new()),
      self.everyone.unwrap_or(SpacePermissionSet::new()),
      post_owner_converted
    ].into_iter()
  }
}

impl IntoIterator for PostPermissions {
  type Item = PostPermissionSet;
  type IntoIter = sp_std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    vec![
      self.none.unwrap_or(PostPermissionSet::new()),
      self.space_owner.unwrap_or(PostPermissionSet::new()),
      self.follower.unwrap_or(PostPermissionSet::new()),
      self.everyone.unwrap_or(PostPermissionSet::new()),
      self.post_owner.unwrap_or(PostPermissionSet::new())
    ].into_iter()
  }
}

impl From<usize> for BuiltinRole {
  fn from(v: usize) -> BuiltinRole {
    match v {
      0 => BuiltinRole::None,
      1 => BuiltinRole::SpaceOwner,
      2 => BuiltinRole::Follower,
      3 => BuiltinRole::Everyone,
      4 => BuiltinRole::PostOwner,
      _ => BuiltinRole::None
    }
  }
}

// -------------------------------------------------------------------------------------------------

// Idea is to iterate BuiltinRole and return IterItem if SpacePermissionSet contains needed value

/*impl BuiltinRole {
  fn get_space_permissions_set_by_role(&self, space_perms: SpacePermissions) -> SpacePermissionSet {
    match self {
      BuiltinRole::None => space_perms.none.unwrap_or(SpacePermissionSet::new()),
      BuiltinRole::SpaceOwner => space_perms.space_owner.unwrap_or(SpacePermissionSet::new()),
      BuiltinRole::Follower => space_perms.follower.unwrap_or(SpacePermissionSet::new()),
      BuiltinRole::Everyone => space_perms.everyone.unwrap_or(SpacePermissionSet::new()),
      BuiltinRole::PostOwner => {
        if let Some(post_owner) = space_perms.post_owner {
          let mut converted_permissions: SpacePermissionSet = SpacePermissionSet::new();

          for permission in post_owner.iter() {
            converted_permissions.insert(permission.clone().into());
          }

          converted_permissions
        } else {
          SpacePermissionSet::new()
        }
      }
    }
  }

  fn get_post_permissions_set_by_role(&self, space_perms: PostPermissions) -> PostPermissionSet {
    match self {
      BuiltinRole::None => space_perms.none.unwrap_or(PostPermissionSet::new()),
      BuiltinRole::SpaceOwner => space_perms.space_owner.unwrap_or(PostPermissionSet::new()),
      BuiltinRole::Follower => space_perms.follower.unwrap_or(PostPermissionSet::new()),
      BuiltinRole::Everyone => space_perms.everyone.unwrap_or(PostPermissionSet::new()),
      BuiltinRole::PostOwner => space_perms.post_owner.unwrap_or(PostPermissionSet::new())
    }
  }
}*/

pub type SpacePermissionSet = BTreeSet<SpacePermission>;

pub type PostPermissionSet = BTreeSet<PostPermission>;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
  type DefaultSpacePermissions: Get<SpacePermissionSet>;
  type DefaultPostPermissions: Get<PostPermissionSet>;
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    const DefaultSpacePermissions: SpacePermissionSet = T::DefaultSpacePermissions::get();
    const DefaultPostPermissions: PostPermissionSet = T::DefaultPostPermissions::get();
  }
}

impl<T: Trait> Module<T> {

  pub fn has_user_a_space_permission(
    space_permissions_context: SpacePermissionsContext,
    permission: SpacePermission,
  ) -> Option<BuiltinRole> {

    // Try to find a permission in space overrides:
    // let mut role_opt = space_permissions_context.space_perms.get(&permission);

    /*
    Implementation idea is to iterate through _Permissions structure and return BuiltinRole by
    index where specified permission was found
    */
    let mut role_opt: Option<BuiltinRole> = None;
    for (builtin_role, space_perms_set) in space_permissions_context.space_perms.into_iter().enumerate() {
      if let Some(_) = space_perms_set.get(&permission) {
        role_opt = Option::from(builtin_role.into());
      }
    }

    // Look into default space permissions,
    // if there is no permission override for this space:
    let default_perms = T::DefaultSpacePermissions::get();
    if role_opt.is_none() {
      role_opt = default_perms.get(&permission);
    }

    Self::has_user_a_role_permission(
      &space_permissions_context,
      None,
      role_opt
    )
  }

  pub fn has_user_a_post_permission(
    space_permissions_context: SpacePermissionsContext,
    post_permissions_context: PostPermissionsContext,
    permission: PostPermission,
  ) -> Option<BuiltinRole> {

    if let Some(built_in_role) = Self::has_user_a_space_permission(
      space_permissions_context.clone(),
      permission.clone().into()
    ) {
      if built_in_role == BuiltinRole::None {
        return Some(built_in_role);
      }
    }

    // Try to find a permission in post/comment overrides:
    let mut role_opt = post_permissions_context.post_perms.get(&permission);

    // Look into default post permissions,
    // if there is no permission override for this post/comment:
    let default_perms = T::DefaultPostPermissions::get();
    if role_opt.is_none() {
      role_opt = default_perms.get(&permission);
    }

    Self::has_user_a_role_permission(
      &space_permissions_context,
      Some(&post_permissions_context),
      role_opt
    )
  }

  fn has_user_a_role_permission(
    space_permissions_context: &SpacePermissionsContext,
    post_permissions_context_opt: Option<&PostPermissionsContext>,
    role_to_check: Option<&BuiltinRole>,
  ) -> Option<BuiltinRole> {

    let is_space_owner = space_permissions_context.is_space_owner;
    let is_follower = space_permissions_context.is_space_follower;

    let mut is_post_owner = false;
    if let Some(post_permissions_context) = post_permissions_context_opt {
      is_post_owner = post_permissions_context.is_post_owner;
    }

    if let Some(role) = role_to_check {
      if *role == BuiltinRole::None ||
        *role == BuiltinRole::SpaceOwner && is_space_owner ||
        *role == BuiltinRole::Follower && (is_space_owner || is_follower) ||
        *role == BuiltinRole::Everyone ||
        *role == BuiltinRole::PostOwner && is_post_owner
      {
        return Some(role.clone());
      }
    }

    None
  }
}