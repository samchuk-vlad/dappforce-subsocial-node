//! Roles pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use pallet_utils::{Trait as UtilsTrait, BalanceOf, Content, SpaceId};
use crate::sp_api_hidden_includes_decl_storage::hidden_include::traits::Currency;
use pallet_spaces::Module as SpacesModule;

const SPACE: SpaceId = 1001;
const ROLE: RoleId = 1;
const SEED: u32 = 0;

fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

fn space_handle_1() -> Option<Vec<u8>> {
    Some(b"Space_Handle".to_vec())
}

fn default_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

fn permission_set_default() -> Vec<SpacePermission> {
    vec![SpacePermission::ManageRoles]
}

fn updated_role_content_ipfs() -> Content {
    Content::IPFS(b"QmZENA8YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDaazhR8".to_vec())
}

fn permission_set_updated() -> Vec<SpacePermission> {
    vec![SpacePermission::ManageRoles, SpacePermission::CreatePosts]
}

benchmarks! {
	_ { }
    // TODO: Remove copy-paste
    create_role {
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpacesModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;

    }: _(RawOrigin::Signed(caller), SPACE, Some(100u32.into()), default_role_content_ipfs(), permission_set_default())
    verify {
        assert!(RoleById::<T>::get(ROLE).is_some());

        // Check whether data in Role structure is correct
        let role = RoleById::<T>::get(ROLE).unwrap();

        assert!(role.updated.is_none());
        assert_eq!(role.space_id, SPACE);
        assert_eq!(role.disabled, false);
        assert_eq!(role.content, self::default_role_content_ipfs());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_default().into_iter())
        );
    }

    update_role {
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpacesModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
        Module::<T>::create_role(origin.clone().into(), SPACE, Some(100u32.into()), default_role_content_ipfs(), permission_set_default())?;

        let role_update: RoleUpdate = RoleUpdate {
            disabled: Some(true),
            content: Some(updated_role_content_ipfs()),
            permissions: Some(
                BTreeSet::from_iter(self::permission_set_updated().into_iter())
            ),
        };

    }: _(origin.clone(), ROLE, role_update)
    verify {
        assert!(RoleById::<T>::get(ROLE).is_some());

        // Check whether data in Role structure is correct
        let role = RoleById::<T>::get(ROLE).unwrap();

        assert!(role.updated.is_some());
        assert_eq!(role.space_id, SPACE);
        assert_eq!(role.disabled, true);
        assert_eq!(role.content, updated_role_content_ipfs());
        assert_eq!(
            role.permissions,
            BTreeSet::from_iter(self::permission_set_updated().into_iter())
        );
    }

    delete_role {
        let caller: T::AccountId = whitelisted_caller();
        let user: T::AccountId = account("user", 0, SEED);
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpacesModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
        Module::<T>::create_role(origin.clone().into(), SPACE, Some(100u32.into()), default_role_content_ipfs(), permission_set_default())?;
        Module::<T>::grant_role(origin.clone().into(), ROLE, vec![User::Account(user.clone())])?;

    }: _(RawOrigin::Signed(caller), ROLE)
    verify {
        assert!(RoleById::<T>::get(ROLE).is_none());
        assert!(UsersByRoleId::<T>::get(ROLE).is_empty());
        assert!(RoleIdsBySpaceId::get(SPACE).is_empty());
        assert!(RoleIdsByUserInSpace::<T>::get(User::Account(user), SPACE).is_empty());
    }

    grant_role {
        let caller: T::AccountId = whitelisted_caller();
        let user: T::AccountId = account("user", 0, SEED);
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpacesModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
        Module::<T>::create_role(origin.clone().into(), SPACE, Some(100u32.into()), default_role_content_ipfs(), permission_set_default())?;

    }: _(RawOrigin::Signed(caller), ROLE, vec![User::Account(user.clone())])
    verify {
        assert_eq!(UsersByRoleId::<T>::get(ROLE), vec![User::Account(user.clone())]);
        assert_eq!(RoleIdsByUserInSpace::<T>::get(User::Account(user.clone()), SPACE), vec![ROLE]);
    }

    revoke_role {
        let caller: T::AccountId = whitelisted_caller();
        let user: T::AccountId = account("user", 0, SEED);
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpacesModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
        Module::<T>::create_role(origin.clone().into(), SPACE, Some(100u32.into()), default_role_content_ipfs(), permission_set_default())?;
        Module::<T>::grant_role(origin.clone().into(), ROLE, vec![User::Account(user.clone())])?;

    }: _(RawOrigin::Signed(caller), ROLE, vec![User::Account(user.clone())])
    verify {
        assert!(UsersByRoleId::<T>::get(ROLE).is_empty());
        assert!(RoleIdsByUserInSpace::<T>::get(User::Account(user), SPACE).is_empty());
    }
}
