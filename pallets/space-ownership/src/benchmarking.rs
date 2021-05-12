//! Space-ownership pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::{vec, prelude::*};
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use pallet_utils::{Trait as UtilsTrait, BalanceOf, Content, SpaceId};
use crate::sp_api_hidden_includes_decl_storage::hidden_include::traits::Currency;
use pallet_spaces::Module as SpaceModule;

const SPACE: SpaceId = 1001;
const SEED: u32 = 0;

fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

fn space_handle_1() -> Option<Vec<u8>> {
    Some(b"Space_Handle".to_vec())
}

benchmarks! {
	_ { }
    // TODO: Remove copy-paste
    transfer_space_ownership {
        let caller: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("user", 0, SEED);
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;

    }: _(RawOrigin::Signed(caller), SPACE, new_owner.clone())
    verify {
        assert_eq!(PendingSpaceOwner::<T>::get(SPACE).unwrap(), new_owner);
    }

    accept_pending_ownership {
        let caller: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("user", 0, SEED);
        let new_owner_origin = RawOrigin::Signed(new_owner.clone());
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
        Module::<T>::transfer_space_ownership(origin.into(), SPACE, new_owner.clone())?;

    }: _(new_owner_origin, SPACE)
    verify {
        let space = SpaceModule::<T>::space_by_id(SPACE).unwrap();
        assert_eq!(space.owner, new_owner);

        assert!(PendingSpaceOwner::<T>::get(SPACE).is_none());
    }

    reject_pending_ownership {
        let caller: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("user", 0, SEED);
        let new_owner_origin = RawOrigin::Signed(new_owner.clone());
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
        Module::<T>::transfer_space_ownership(origin.into(), SPACE, new_owner)?;

    }: _(new_owner_origin, SPACE)
    verify {
         let space = SpaceModule::<T>::space_by_id(SPACE).unwrap();
        assert_eq!(space.owner, caller);

        assert!(PendingSpaceOwner::<T>::get(SPACE).is_none());
    }

}
