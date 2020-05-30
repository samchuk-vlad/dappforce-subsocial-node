use super::*;

use frame_support::{dispatch::{DispatchResult, DispatchError}};

impl<T: Trait> Module<T> {

  pub fn ensure_role_manager(account: T::AccountId, space_id: SpaceId) -> DispatchResult {
    Self::ensure_user_has_space_permission_with_load_space(
      User::Account(account),
      space_id,
      SpacePermission::ManageRoles,
      Error::<T>::NoPermissionToManageRoles.into()
    )
  }

  fn ensure_user_has_space_permission_with_load_space(
    user: User<T::AccountId>,
    space_id: SpaceId,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {

    let space = T::Spaces::get_space(space_id)?;

    let mut is_owner = false;
    let mut is_follower = false;

    match &user {
      User::Account(account) => {
        is_owner = *account == space.owner;

        // No need to check if a user is follower, if they already are an owner:
        is_follower = is_owner || T::Spaces::is_space_follower(account.clone(), space_id);
      }
      User::Space(_) => (/* Not implemented yet. */),
    }

    Self::ensure_user_has_space_permission(
      user,
      space_id,
      is_owner,
      is_follower,
      space.permissions,
      permission,
      error
    )
  }

  fn ensure_user_has_space_permission(
    user: User<T::AccountId>,
    space_id: SpaceId,
    is_owner: bool,
    is_follower: bool,
    space_perms: SpacePermissions,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {

    if Permissions::<T>::has_user_a_space_permission(
      is_owner,
      is_follower,
      space_perms,
      permission.clone(),
    ) {
      return Ok(());
    }

    Self::has_permission_in_space_roles(
      user,
      space_id,
      permission,
      error
    )
  }

  fn ensure_user_has_post_permission(
    user: User<T::AccountId>,
    space_id: SpaceId,
    is_owner: bool,
    is_follower: bool,
    post_perms: PostPermissions,
    space_perms: SpacePermissions,
    permission: PostPermission,
    error: DispatchError,
  ) -> DispatchResult {

    if Permissions::<T>::has_user_a_post_permission(
      is_owner,
      is_follower,
      post_perms,
      space_perms,
      permission.clone(),
    ) {
      return Ok(());
    }

    Self::has_permission_in_space_roles(
      user,
      space_id,
      permission.into(),
      error
    )
  }

  fn has_permission_in_space_roles(
    user: User<T::AccountId>,
    space_id: SpaceId,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {

    let role_ids = Self::in_space_role_ids_by_user((user, space_id));

    for role_id in role_ids {
      if let Some(role) = Self::role_by_id(role_id) {
        if role.disabled {
          continue;
        }

        let mut is_expired = false;
        if let Some(expires_at) = role.expires_at {
          if expires_at <= <system::Module<T>>::block_number() {
            is_expired = true;
          }
        }

        if !is_expired && role.permissions.contains(&permission) {
          return Ok(());
        }
      }
    }

    Err(error)
  }
}

impl<T: Trait> Role<T> {

  pub fn new(
    created_by: T::AccountId,
    space_id: SpaceId,
    time_to_live: Option<T::BlockNumber>,
    ipfs_hash: Option<Vec<u8>>,
    permissions: BTreeSet<SpacePermission>,
  ) -> Result<Self, DispatchError> {

    let role_id = Module::<T>::next_role_id();

    let mut expires_at: Option<T::BlockNumber> = None;
    if let Some(ttl) = time_to_live {
      expires_at = Some(ttl + <system::Module<T>>::block_number());
    }

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

    Ok(new_role)
  }

  pub fn set_disabled(&mut self, disable: bool) -> DispatchResult {
    if self.disabled && disable {
      return Err(Error::<T>::RoleAlreadyDisabled.into());
    } else if !self.disabled && !disable {
      return Err(Error::<T>::RoleAlreadyEnabled.into());
    }

    self.disabled = disable;

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

impl<T: Trait> PermissionChecker for Module<T> {
  type AccountId = T::AccountId;
  type SpaceId = SpaceId;

  fn ensure_user_has_space_permission(
    user: User<Self::AccountId>,
    space_id: Self::SpaceId,
    is_owner: bool,
    is_follower: bool,
    space_perms: SpacePermissions,
    permission: SpacePermission,
    error: DispatchError,
  ) -> DispatchResult {

    Self::ensure_user_has_space_permission(
      user,
      space_id,
      is_owner,
      is_follower,
      space_perms,
      permission,
      error
    )
  }

  fn ensure_user_has_post_permission(
    user: User<T::AccountId>,
    space_id: SpaceId,
    is_owner: bool,
    is_follower: bool,
    post_perms: PostPermissions,
    space_perms: SpacePermissions,
    permission: PostPermission,
    error: DispatchError,
  ) -> DispatchResult {

    Self::ensure_user_has_post_permission(
      user,
      space_id,
      is_owner,
      is_follower,
      post_perms,
      space_perms,
      permission,
      error
    )
  }
}
