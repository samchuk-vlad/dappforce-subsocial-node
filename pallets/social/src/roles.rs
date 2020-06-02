use super::*;

use frame_support::{dispatch::{DispatchResult, DispatchError}};
use pallet_permissions::{SpacePermissionsContext, PostPermissionsContext};

impl<T: Trait> Module<T> {

  pub fn ensure_account_has_space_permission(
    account: T::AccountId,
    space: &Space<T>,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {

    let is_owner = space.is_owner(&account);
    let is_follower = Self::space_followed_by_account((account.clone(), space.id));

    T::Roles::ensure_account_has_space_permission(
      account,
      SpacePermissionsContext {
        space_id: space.id,
        is_space_owner: is_owner,
        is_space_follower: is_follower,
        space_perms: space.permissions.clone()
      },
      permission,
      error
    )
  }

  pub fn ensure_account_has_post_permission(
    account: T::AccountId,
    post: &Post<T>,
    space: &Space<T>,
    permission: PostPermission,
    error: DispatchError,
  ) -> DispatchResult {

    let is_post_owner = post.is_owner(&account);
    let is_space_owner = space.is_owner(&account);
    let is_follower = Self::space_followed_by_account((account.clone(), space.id));

    T::Roles::ensure_account_has_post_permission(
      account,
      SpacePermissionsContext {
        space_id: space.id,
        is_space_owner,
        is_space_follower: is_follower,
        space_perms: space.permissions.clone()
      },
      PostPermissionsContext {
        is_post_owner,
        post_perms: post.permissions.clone()
      },
      permission,
      error
    )
  }
}