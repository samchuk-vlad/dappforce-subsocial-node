#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchResult, DispatchError};

use pallet_permissions::{
  SpacePermission,
  SpacePermissions,
  SpacePermissionsContext
};
use pallet_utils::User;

/// Minimal set of fields from Space struct that are required by roles pallet.
pub struct SpaceForRoles<AccountId> {
  pub owner: AccountId,
  pub permissions: Option<SpacePermissions>,
}

pub trait SpaceForRolesProvider {
  type AccountId;
  type SpaceId;

  fn get_space(id: Self::SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError>;

  fn is_space_follower(account: Self::AccountId, space_id: Self::SpaceId) -> bool;
}

pub trait PermissionChecker {
  type AccountId;

  fn ensure_user_has_space_permission(
    user: User<Self::AccountId>,
    space_perms_context: SpacePermissionsContext,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult;

  fn ensure_account_has_space_permission(
    account: Self::AccountId,
    space_perms_context: SpacePermissionsContext,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {

    Self::ensure_user_has_space_permission(
      User::Account(account),
      space_perms_context,
      permission,
      error
    )
  }
}