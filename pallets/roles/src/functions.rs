use super::*;

use frame_support::{dispatch::{DispatchResult, DispatchError}};

impl<T: Trait> Module<T> {
  fn has_permission_in_override(permissions_opt: Option<BTreeSet<SpacePermission>>, permission: &SpacePermission) -> bool {
    if let Some(permissions) = permissions_opt {
      if permissions.contains(permission) {
        return true;
      }
    }

    false
  }

  pub fn ensure_user_has_space_permission(
    user: User<T::AccountId>, space: &Space<T>, permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {
    // TODO: maybe make function as impl for Space?
    // TODO: check default permissions only if no overrides found
    // TODO: think on checks priority
    // TODO: maybe move permissions iterations/common functions into pallet-permissions?

    match &user {
      User::Account(account_id) => {
        if space.owner == *account_id {
          return Ok(());
        }
      }
      User::Space(_) => (),
    }

    if Self::has_permission_in_override(space.everyone_permissions.clone(), &permission) ||
      Self::has_permission_in_override(space.follower_permissions.clone(), &permission) {
      return Ok(());
    }

    let default_everyone_permissions = <T as PermissionsTrait>::DefaultEveryoneSpacePermissions::get();
    let default_follower_permissions = <T as PermissionsTrait>::DefaultFollowerSpacePermissions::get();

    if default_everyone_permissions.contains(&permission) ||
      default_follower_permissions.contains(&permission) {
      return Ok(());
    }

    let role_ids = Self::in_space_role_ids_by_user((user, space.id));

    for role_id in role_ids {
      let role = Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?;

      let mut expired = false;
      if let Some(expires_at) = role.expires_at {
        if expires_at <= <system::Module<T>>::block_number() {
          expired = true;
        }
      }

      if !role.disabled && !expired && role.permissions.contains(&permission) {
        return Ok(());
      }
    }

    Err(error)
  }
}

impl<T: Trait> Role<T> {
  pub fn new(
    created_by: T::AccountId,
    space_id: SpaceId,
    ipfs_hash: Option<Vec<u8>>,
    expires_at: Option<T::BlockNumber>,
    permissions: BTreeSet<SpacePermission>,
  ) -> Result<(Self, RoleId), DispatchError> {
    let role_id = Module::<T>::next_role_id();
    let new_role = Role::<T> {
      created: WhoAndWhen::new(created_by),
      updated: None,
      id: role_id,
      space_id,
      disabled: false,
      expires_at,
      ipfs_hash,
      permissions,
    };

    let next_role_id = role_id.checked_add(1).ok_or(Error::<T>::OverflowCreatingNewRole)?;

    Ok((new_role, next_role_id))
  }

  pub fn change_disabled_state(&mut self, new_disabled_state: bool) -> DispatchResult {
    if self.disabled && new_disabled_state {
      return Err(Error::<T>::RoleIsNotEnabled.into());
    } else if !self.disabled && !new_disabled_state {
      return Err(Error::<T>::RoleIsAlreadyEnabled.into());
    }

    self.disabled = new_disabled_state;

    Ok(())
  }

  pub fn revoke_from_users(&self, users: Vec<User<T::AccountId>>) {
    for user in users.iter() {
      let role_idx_by_user_opt = Module::<T>::in_space_role_ids_by_user((&user, self.space_id)).iter()
        .position(|x| { *x == self.id });

      if let Some(role_idx) = role_idx_by_user_opt {
        <InSpaceRoleIdsByUser<T>>::mutate((user, self.space_id), |n| { n.swap_remove(role_idx) });
      }
    }
  }
}
