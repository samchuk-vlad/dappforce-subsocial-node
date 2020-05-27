#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

pub mod functions;
// mod tests;

use sp_std::prelude::*;
use sp_std::collections::btree_set::BTreeSet;
use codec::{Encode, Decode};
use frame_support::{
  decl_module, decl_storage, decl_event, decl_error,/* ensure,*/
  traits::Get
};
use sp_runtime::RuntimeDebug;
use system::ensure_signed;

use pallet_utils::{Module as Utils, WhoAndWhen, User};
use pallet_social::{Module as Social, Space, SpaceId};
use pallet_permissions::SpacePermission;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Role<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub id: RoleId,
  pub space_id: SpaceId,
  pub disabled: bool,
  pub ipfs_hash: Vec<u8>,
  pub permissions: BTreeSet<SpacePermission>
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct RoleUpdate {
  pub disabled: Option<bool>,
  pub ipfs_hash: Option<Vec<u8>>,
  pub permissions: Option<BTreeSet<SpacePermission>>,
}

type RoleId = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_social::Trait /*pallet-utils is in pallet's-social Trait*/ {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

  type DefaultEveryoneSpacePermissions: Get<BTreeSet<SpacePermission>>;

  type DefaultFollowerSpacePermissions: Get<BTreeSet<SpacePermission>>;
}

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
    /// No roles found for this User on specified Space
    NoAnyRolesForUserOnSpace,
    /// No roles provided when trying to create a new Role
    NoPermissionsProvided,
    /// No users provided when trying to grant them a Role
    NoUsersProvided,
    /// Trying to disable the role that is not enabled (or disabled as by default)
    RoleIsNotEnabled,

    // TODO: remove this error, when Space is implemented as a profile
    /// User must be an Account
    UserIsNotAnAccount,
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
    const DefaultEveryoneSpacePermissions: BTreeSet<SpacePermission> = T::DefaultEveryoneSpacePermissions::get();

    const DefaultFollowerSpacePermissions: BTreeSet<SpacePermission> = T::DefaultFollowerSpacePermissions::get();

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

      Self::ensure_user_has_space_permission(
        User::Account(who.clone()), &space, SpacePermission::ManageRoles,
        Error::<T>::NoPermissionToManageRoles.into()
      )?;

      let role_id = Self::next_role_id();
      let new_role = Role::<T> {
        created: WhoAndWhen::new(who.clone()),
        updated: None,
        id: role_id,
        space_id,
        disabled: false,
        ipfs_hash,
        permissions: permissions_set
      };

      let next_role_id = role_id.checked_add(1).ok_or(Error::<T>::OverflowCreatingNewRole)?;
      NextRoleId::put(next_role_id);

      <RoleById<T>>::insert(role_id, new_role);
      RoleIdsBySpaceId::mutate(space_id, |role_ids| { role_ids.push(role_id) });

      Self::deposit_event(RawEvent::RoleCreated(who, space_id, role_id));
    }

    /// Update an existing role on specified space.
    /// It is possible to either update roles by overriding existing roles,
    /// or update IPFS hash or both.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn update_role(origin, _role_id: RoleId, _update: RoleUpdate) {
      let _who = ensure_signed(origin)?;
    }

    /// Delete the role from all associated storage items.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn delete_role(origin, _role_id: RoleId, _update: RoleUpdate) {
      let _who = ensure_signed(origin)?;
    }

    /// Grant the role from the list of users.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn grant_role(origin, role_id: RoleId, users_vec: Vec<User<T::AccountId>>) {
      let who = ensure_signed(origin)?;

      let mut users_set: BTreeSet<User<T::AccountId>> = BTreeSet::new();
      if users_vec.is_empty() {
        return Err(Error::<T>::NoUsersProvided.into());
      }
      users_vec.iter().for_each(|u| { users_set.insert(u.clone()); });

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
    pub fn revoke_role(origin, _role_id: RoleId, _users: Vec<User<T::AccountId>>) {
      let _who = ensure_signed(origin)?;
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

      if role.disabled {
        return Err(Error::<T>::RoleIsNotEnabled.into());
      }
      role.disabled = true;

      <RoleById<T>>::insert(role_id, role);
      Self::deposit_event(RawEvent::RoleDisabled(who, role_id));
    }

    /// Enable the role. Should throw an error if the role is not disabled.
    /// Only the space owner or an user with `ManageRoles` permission can execute this extrinsic.
    pub fn enable_role(origin, _role_id: RoleId) {
      let _who = ensure_signed(origin)?;
    }
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId {
    RoleCreated(AccountId, SpaceId, RoleId),
    RoleGranted(AccountId, RoleId, Vec<User<AccountId>>),
    RoleDisabled(AccountId, RoleId),
  }
);
