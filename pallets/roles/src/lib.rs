#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

pub mod functions;
// mod tests;

use sp_std::{
  prelude::*,
  collections::btree_set::BTreeSet
};
use codec::{Encode, Decode};
use frame_support::{
  decl_module, decl_storage, decl_event, decl_error, ensure,
  traits::Get
};
use sp_runtime::RuntimeDebug;
use system::ensure_signed;

use pallet_utils::{Module as Utils, WhoAndWhen, User};
use pallet_social::{Module as Social, Space, SpaceId};
use pallet_permissions::{SpacePermission, Trait as PermissionsTrait};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Role<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub id: RoleId,
  pub space_id: SpaceId,
  pub disabled: bool,
  pub expires_at:  Option<T::BlockNumber>,
  pub ipfs_hash: Option<Vec<u8>>,
  pub permissions: BTreeSet<SpacePermission>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct RoleUpdate {
  pub ipfs_hash: Option<Option<Vec<u8>>>,
  pub permissions: Option<BTreeSet<SpacePermission>>,
}

type RoleId = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_social::Trait + pallet_permissions::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

  type MaxUsersToProcessPerDeleteRole: Get<u16>;
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId {
    RoleCreated(AccountId, SpaceId, RoleId),
    RoleUpdated(AccountId, RoleId),
    RoleGranted(AccountId, RoleId, Vec<User<AccountId>>),
    RoleEnabled(AccountId, RoleId),
    RoleDisabled(AccountId, RoleId),
    RoleDeleted(AccountId, RoleId),
  }
);

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Space was not found by id
    SpaceNotFound,

    /// Role was not found by id
    RoleNotFound,
    /// RoleId counter storage overflowed
    OverflowCreatingNewRole,
    /// Account has not permission to manage roles on this space
    NoPermissionToManageRoles,
    /// Nothing to update in role
    NoUpdatesInRole,
    /// There's too many users assigned for this role to delete it
    TooManyUserAssignedToDeleteRole,

    /// No roles found for this User on specified Space
    NoAnyRolesForUserOnSpace,
    /// No roles provided when trying to create a new Role
    NoPermissionsProvided,
    /// No users provided when trying to grant them a Role
    NoUsersProvided,
    /// Trying to disable the role that is not enabled (or disabled as by default)
    RoleIsNotEnabled,
    /// Trying to enable the role that is already enabled
    RoleIsAlreadyEnabled,
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as PermissionsModule {
    /// Get role details by ids id.
    pub RoleById get(fn role_by_id): map RoleId => Option<Role<T>>;

    /// A list of all account ids and space ids that have this role.
    pub UsersByRoleId get(fn users_by_role_id): map RoleId => Vec<User<T::AccountId>>;

    /// A list of all role ids available in this space.
    pub RoleIdsBySpaceId get(fn role_ids_by_space_id): map SpaceId => Vec<RoleId>;

    /// A list of all role ids granted to this user (either account of space) within this space.
    pub InSpaceRoleIdsByUser get(fn in_space_role_ids_by_user): map (User<T::AccountId>, SpaceId) => Vec<RoleId>;

    /// Next available RoleId
    pub NextRoleId get(fn next_role_id): RoleId = 1;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    const MaxUsersToProcessPerDeleteRole: u16 = T::MaxUsersToProcessPerDeleteRole::get();

    // Initializing events
    // this is needed only if you are using events in your pallet
    fn deposit_event() = default;

    /// Create a new role within this space with the list of particular roles.
    /// `ipfs_hash` points to the off-chain content with such role info as name, description, color.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn create_role(origin, space_id: SpaceId, ipfs_hash: Vec<u8>, permissions: Vec<SpacePermission>) {
      let who = ensure_signed(origin)?;

      Utils::<T>::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let mut permissions_set: BTreeSet<SpacePermission> = BTreeSet::new();
      if permissions.is_empty() {
        return Err(Error::<T>::NoPermissionsProvided.into());
      }
      permissions.iter().for_each(|p| { permissions_set.insert(p.clone()); });

      // TODO: maybe add impl for space instead of `does_user_has_space_permission`?
      let space: Space<T> = Social::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?;

      // TODO: what if role is created by User::Space?
      Self::ensure_user_has_space_permission(
        User::Account(who.clone()), &space, SpacePermission::ManageRoles,
        Error::<T>::NoPermissionToManageRoles.into()
      )?;

      let (new_role, role_id) = Role::<T>::new(who.clone(), space_id, Some(ipfs_hash), None, permissions_set)?;

      NextRoleId::put(role_id);

      <RoleById<T>>::insert(new_role.id, new_role.clone());
      RoleIdsBySpaceId::mutate(space_id, |role_ids| { role_ids.push(new_role.id) });

      Self::deposit_event(RawEvent::RoleCreated(who, space_id, role_id));
    }

    /// Update an existing role on specified space.
    /// It is possible to either update roles by overriding existing roles,
    /// or update IPFS hash or both.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn update_role(origin, role_id: RoleId, update: RoleUpdate) {
      let who = ensure_signed(origin)?;

      let has_updates =
        update.ipfs_hash.is_some() ||
        update.permissions.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInRole);

      let mut role = Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?;
      let space: Space<T> = Social::space_by_id(role.space_id).ok_or(Error::<T>::SpaceNotFound)?;

      Self::ensure_user_has_space_permission(
        User::Account(who.clone()), &space, SpacePermission::ManageRoles,
        Error::<T>::NoPermissionToManageRoles.into()
      )?;

      let mut fields_updated = 0;

      if let Some(ipfs_hash_opt) = update.ipfs_hash {
        if ipfs_hash_opt != role.ipfs_hash {
          if let Some(ipfs_hash) = ipfs_hash_opt.clone() {
            Utils::<T>::is_ipfs_hash_valid(ipfs_hash)?;
          }

          role.ipfs_hash = ipfs_hash_opt;
          fields_updated += 1;
        }
      }

      if let Some(permissions) = update.permissions {
        let permissions_diff: Vec<_> = role.permissions.difference(&permissions).cloned().collect();

        if !permissions_diff.is_empty() {
          role.permissions = permissions;
          fields_updated += 1;
        }
      }

      if fields_updated > 0 {
        role.updated = Some(WhoAndWhen::<T>::new(who.clone()));

        <RoleById<T>>::insert(role_id, role);
        Self::deposit_event(RawEvent::RoleUpdated(who, role_id));
      }
    }

    /// Delete the role from all associated storage items.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn delete_role(origin, role_id: RoleId) {
      let who = ensure_signed(origin)?;

      let role = Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?;
      let space: Space<T> = Social::space_by_id(role.space_id).ok_or(Error::<T>::SpaceNotFound)?;

      Self::ensure_user_has_space_permission(
        User::Account(who.clone()), &space, SpacePermission::ManageRoles,
        Error::<T>::NoPermissionToManageRoles.into()
      )?;

      let users = Self::users_by_role_id(role_id);
      if users.len() > T::MaxUsersToProcessPerDeleteRole::get() as usize {
        return Err(Error::<T>::TooManyUserAssignedToDeleteRole.into());
      }

      let role_idx_by_space_opt = Self::role_ids_by_space_id(space.id).iter()
        .position(|x| { *x == role_id });

      if let Some(role_idx) = role_idx_by_space_opt {
        RoleIdsBySpaceId::mutate(space.id, |n| { n.swap_remove(role_idx) });
      }

      role.revoke_from_users(users);

      <RoleById<T>>::remove(role_id);
      <UsersByRoleId<T>>::remove(role_id);

      Self::deposit_event(RawEvent::RoleDeleted(who, role_id));
    }

    /// Grant the role from the list of users.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn grant_role(origin, role_id: RoleId, users: Vec<User<T::AccountId>>) {
      let who = ensure_signed(origin)?;

      let users_set: BTreeSet<User<T::AccountId>> = Self::users_vec_to_btree_set(users)?;

      let role = Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?;
      let space: Space<T> = Social::space_by_id(role.space_id).ok_or(Error::<T>::SpaceNotFound)?;

      Self::ensure_user_has_space_permission(
        User::Account(who.clone()), &space, SpacePermission::ManageRoles,
        Error::<T>::NoPermissionToManageRoles.into()
      )?;

      for user in users_set.iter() {
        if !Self::users_by_role_id(role_id).contains(&user) {
          <UsersByRoleId<T>>::mutate(role_id, |users| { users.push(user.clone()); });
        }
        if !Self::in_space_role_ids_by_user((user.clone(), space.id)).contains(&role_id) {
          <InSpaceRoleIdsByUser<T>>::mutate((user.clone(), space.id), |roles| { roles.push(role_id); })
        }
      }

      Self::deposit_event(RawEvent::RoleGranted(who, role_id, users_set.iter().cloned().collect()));
    }

    /// Revoke the role from the list of users.
    /// Only the space owner, an user with `ManageRoles` permission or an user that has this role can execute this extrinsic.
    pub fn revoke_role(origin, role_id: RoleId, users: Vec<User<T::AccountId>>) {
      let who = ensure_signed(origin)?;

      let role = Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?;
      let space: Space<T> = Social::space_by_id(role.space_id).ok_or(Error::<T>::SpaceNotFound)?;

      Self::ensure_user_has_space_permission(
        User::Account(who.clone()), &space, SpacePermission::ManageRoles,
        Error::<T>::NoPermissionToManageRoles.into()
      )?;

      role.revoke_from_users(users.clone());

      Self::deposit_event(RawEvent::RoleGranted(who, role_id, users));
    }

    /// Disable the role. If the role is disabled, their roles should not be taken into account.
    /// Should throw an error if the role is not enabled.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn disable_role(origin, role_id: RoleId) {
      let who = ensure_signed(origin)?;

      let mut role = Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?;
      let space: Space<T> = Social::space_by_id(role.space_id).ok_or(Error::<T>::SpaceNotFound)?;

      Self::ensure_user_has_space_permission(
        User::Account(who.clone()), &space, SpacePermission::ManageRoles,
        Error::<T>::NoPermissionToManageRoles.into()
      )?;

      role.change_disabled_state(true)?;

      <RoleById<T>>::insert(role_id, role);
      Self::deposit_event(RawEvent::RoleDisabled(who, role_id));
    }

    /// Enable the role. Should throw an error if the role is not disabled.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn enable_role(origin, role_id: RoleId) {
      let who = ensure_signed(origin)?;

      let mut role = Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?;
      let space: Space<T> = Social::space_by_id(role.space_id).ok_or(Error::<T>::SpaceNotFound)?;

      Self::ensure_user_has_space_permission(
        User::Account(who.clone()), &space, SpacePermission::ManageRoles,
        Error::<T>::NoPermissionToManageRoles.into()
      )?;

      role.change_disabled_state(false)?;

      <RoleById<T>>::insert(role_id, role);
      Self::deposit_event(RawEvent::RoleEnabled(who, role_id));
    }
  }
}
