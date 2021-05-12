//! Space-follows pallet benchmarking.

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
    follow_space {
        let caller: T::AccountId = whitelisted_caller();
        let follower: T::AccountId = account("user", 0, SEED);
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;

    }: _(RawOrigin::Signed(follower.clone()), SPACE)
    verify {
        // assert_eq!(pallet_spaces::SpaceById::get(SPACE).unwrap().followers_count, 2);
        assert_eq!(SpacesFollowedByAccount::<T>::get(follower.clone()), vec![SPACE]);
        assert_eq!(SpaceFollowers::<T>::get(SPACE), vec![caller, follower.clone()]);
        assert_eq!(SpaceFollowedByAccount::<T>::get((follower, SPACE)), true);
    }

    unfollow_space {
        let caller: T::AccountId = whitelisted_caller();
        let follower: T::AccountId = account("user", 0, SEED);
        let follower_origin = RawOrigin::Signed(follower.clone());
        let origin = RawOrigin::Signed(caller.clone());

        <T as UtilsTrait>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        SpaceModule::<T>::create_space(origin.clone().into(), None, space_handle_1(), space_content_ipfs(), None)?;
        Module::<T>::follow_space(follower_origin.clone().into(), SPACE)?;
        
    }: _(follower_origin, SPACE)
    verify {
        assert!(SpacesFollowedByAccount::<T>::get(follower.clone()).is_empty());
        assert_eq!(SpaceFollowers::<T>::get(SPACE), vec![caller]);
    }
}
