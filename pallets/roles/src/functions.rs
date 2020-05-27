use super::*;

use frame_support::{dispatch::{DispatchResult, DispatchError}};

impl<T: Trait> Module<T> {
  fn iter_permissions_set(permissions_opt: Option<BTreeSet<SpacePermission>>, permission: &SpacePermission) -> bool {
    if let Some(permissions) = permissions_opt {
      if permissions.iter().any(|p| p == permission) {
        return true;
      }
    }

    false
  }

  pub fn ensure_user_has_space_permission(
    user: User<T::AccountId>, space: &Space<T>, permission: SpacePermission,
    error: DispatchError
  ) -> DispatchResult {
    // TODO: maybe make function as impl for Space?
    // TODO: think about root roles (e.g. ManagePosts is a root role of CreatePost, ..., DeletePost)

    match &user {
      User::Account(account_id) => {
        if space.owner == *account_id {
          return Ok(());
        }
      },
      User::Space(_) => return Err(Error::<T>::UserIsNotAnAccount.into())
    }

    if Self::iter_permissions_set(space.everyone_permissions.clone(), &permission)
      || Self::iter_permissions_set(space.follower_permissions.clone(), &permission) {
      return Ok(());
    }

    // TODO: if there are no overrides, check default roles

    let role_ids = Self::in_space_role_ids_by_user((user, space.id));

    for role_id in role_ids {
      let role = Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?;
      if !role.disabled && role.permissions.contains(&permission) {
        return Ok(());
      }
    }

    Err(error)
  }
}
