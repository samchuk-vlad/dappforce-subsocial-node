//! Faucets pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::{vec};
use frame_system::{RawOrigin};
// use frame_support::ensure;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use sp_runtime::traits::Bounded;
use crate::sp_api_hidden_includes_decl_storage::hidden_include::traits::Currency;

const SEED: u32 = 0;

benchmarks! {
	_ { }

    add_faucet {
        let faucet: T::AccountId = account("user", 0, SEED);

        <T as Trait>::Currency::make_free_balance_be(&faucet, BalanceOf::<T>::max_value());

    }: _(RawOrigin::Root, faucet.clone(), 100u32.into(), 100u32.into(), 50u32.into())
    verify {
        let faucet_account: Faucet<T> = FaucetByAccount::<T>::get(faucet).unwrap();

        assert_eq!(faucet_account.enabled, true);
        assert_eq!(faucet_account.period, 100u32.into());
        assert_eq!(faucet_account.period_limit, 100u32.into());
        assert_eq!(faucet_account.drip_limit, 50u32.into());
        assert_eq!(faucet_account.next_period_at, 0u32.into());
        assert_eq!(faucet_account.dripped_in_current_period, 0u32.into());
    }

    update_faucet {
        let faucet: T::AccountId = account("user", 0, SEED);

        <T as Trait>::Currency::make_free_balance_be(&faucet, BalanceOf::<T>::max_value());

        Module::<T>::add_faucet(RawOrigin::Root.into(), faucet.clone(), 100u32.into(), 100u32.into(), 50u32.into())?;

    }: _(RawOrigin::Root, faucet.clone(), FaucetUpdate {
            enabled: None,
            period: Some(7_200u32.into()),
            period_limit: Some(90u32.into()),
            drip_limit: Some(40u32.into())
        })
    verify {
        let faucet_account: Faucet<T> = FaucetByAccount::<T>::get(faucet).unwrap();

        assert_eq!(faucet_account.period, 7_200u32.into());
        assert_eq!(faucet_account.period_limit, 90u32.into());
        assert_eq!(faucet_account.drip_limit, 40u32.into());
    }

    remove_faucets {
        let faucet: T::AccountId = account("user", 0, SEED);

        <T as Trait>::Currency::make_free_balance_be(&faucet, BalanceOf::<T>::max_value());

        Module::<T>::add_faucet(RawOrigin::Root.into(), faucet.clone(), 100u32.into(), 100u32.into(), 50u32.into())?;

    }: _(RawOrigin::Root, vec![faucet.clone()])
    verify {
        assert!(FaucetByAccount::<T>::get(faucet).is_none())
    }

     drip {
        let caller: T::AccountId = whitelisted_caller();
        let faucet: T::AccountId = account("user", 0, SEED);
        let faucet_origin = RawOrigin::Signed(faucet.clone());
        let origin = RawOrigin::Signed(caller.clone());

        <T as Trait>::Currency::make_free_balance_be(&faucet, BalanceOf::<T>::max_value());
        Module::<T>::add_faucet(RawOrigin::Root.into(), faucet.clone(), 1000u32.into(), 100000u32.into(), 10000u32.into())?;

    }: _(faucet_origin, caller.clone(), 1u32.into())
    verify {
        assert_eq!(pallet_balances::Module::<T>::free_balance(caller), 1u32.into());
    }
}
