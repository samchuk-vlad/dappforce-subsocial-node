use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_std::collections::btree_set::BTreeSet;
use pallet_permissions::SpacePermission;
use crate::User;

/// Minimal set of fields from Space struct that are required by roles pallet.
pub struct SpaceForRoles<AccountId> {
  pub owner: AccountId,
  pub everyone_permissions: Option<BTreeSet<SpacePermission>>,
  pub follower_permissions: Option<BTreeSet<SpacePermission>>,
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
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult;

  fn ensure_account_has_space_permission(
    account: Self::AccountId,
    space_id: Self::SpaceId,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {
    Self::ensure_user_has_space_permission(
      User::Account(account),
      space_id,
      permission,
      error
    )
  }
}