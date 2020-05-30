#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchResult, DispatchError};

use pallet_permissions::{
  SpacePermission,
  SpacePermissions,
  PostPermission,
  PostPermissions
};
use pallet_utils::User;

/// Minimal set of fields from Space struct that are required by roles pallet.
pub struct SpaceForRoles<AccountId> {
  pub owner: AccountId,
  pub permissions: SpacePermissions,
}

pub trait SpaceForRolesProvider {
  type AccountId;
  type SpaceId;

  fn get_space(id: Self::SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError>;

  fn is_space_follower(account: Self::AccountId, space_id: Self::SpaceId) -> bool;
}

pub trait PermissionChecker {
  type AccountId;
  type SpaceId;

  fn ensure_user_has_space_permission(
    user: User<Self::AccountId>,
    space_id: Self::SpaceId,
    is_owner: bool,
    is_follower: bool,
    space_perms: SpacePermissions,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult;

  fn ensure_user_has_post_permission(
    user: User<Self::AccountId>,
    space_id: Self::SpaceId,
    is_post_owner: bool,
    is_space_owner: bool,
    is_follower: bool,
    post_perms: PostPermissions,
    space_perms: SpacePermissions,
    permission: PostPermission,
    error: DispatchError
  ) -> DispatchResult;

  fn ensure_account_has_space_permission(
    account: Self::AccountId,
    space_id: Self::SpaceId,
    is_owner: bool,
    is_follower: bool,
    space_perms: SpacePermissions,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {

    Self::ensure_user_has_space_permission(
      User::Account(account),
      space_id,
      is_owner,
      is_follower,
      space_perms,
      permission,
      error
    )
  }

  fn ensure_account_has_post_permission(
    account: Self::AccountId,
    space_id: Self::SpaceId,
    is_post_owner: bool,
    is_space_owner: bool,
    is_follower: bool,
    post_perms: PostPermissions,
    space_perms: SpacePermissions,
    permission: PostPermission,
    error: DispatchError
  ) -> DispatchResult {

    Self::ensure_user_has_post_permission(
      User::Account(account),
      space_id,
      is_post_owner,
      is_space_owner,
      is_follower,
      post_perms,
      space_perms,
      permission,
      error
    )
  }
}