//! # Faucets Module
//!
//! The Faucets module allows a root key (sudo) to add accounts (faucets) that are eligible
//! to drip free tokens to other accounts (recipients).
//! 
//! Currently, only sudo account can add, update and remove faucets.
//! But this can be changed in the future to allow anyone else
//! to set up new faucets for their needs.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, ExistenceRequirement, Get},
    weights::Pays,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{
    collections::btree_set::BTreeSet,
    iter::FromIterator,
    prelude::*,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// TODO maybe merge FaucetSettings + FaucetState => Faucet? this will reduce storage reads.

// TODO allow anyone account to add, update, remove faucets, not only sudo.

// TODO add 'created' to FaucetSettings so we can check owner?

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetState<BlockNumber, Balance> {
    next_period_at: BlockNumber,
    dripped_in_current_period: Balance,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetSettings<BlockNumber, Balance> {
    period: Option<BlockNumber>,
    period_limit: Balance,
    drip_limit: Balance,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetSettingsUpdate<BlockNumber, Balance> {
    period: Option<Option<BlockNumber>>,
    period_limit: Option<Balance>,
    drip_limit: Option<Balance>,
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as FaucetModule {

		pub SettingsByFaucet get(fn settings_by_faucet):
			map hasher(twox_64_concat) T::AccountId
			=> Option<FaucetSettings<T::BlockNumber, BalanceOf<T>>>;

        pub FaucetDropsInfo get(fn faucet_drops_info):
            map hasher(twox_64_concat) T::AccountId
            => Option<FaucetState<T::BlockNumber, BalanceOf<T>>>;

	    pub TotalFaucetDropsByAccount get(fn total_faucet_drops_by_account): double_map
	        hasher(twox_64_concat) T::AccountId, // Faucet account
	        hasher(twox_64_concat) T::AccountId  // User account
	        => BalanceOf<T>;
	}
}

decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>
	{
		FaucetAdded(AccountId),
		FaucetUpdated(AccountId),
		FaucetsRemoved(Vec<AccountId>),
		TokensDropped(
			AccountId, // Faucet
			AccountId, // Recipient
			Balance    // Amount dripped
		),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		FaucetNotFound,
		FaucetAlreadyAdded,
		NoFreeBalanceOnFaucet,
        
        NoUpdatesProvided,
        NoFaucetsProvided,

		ZeroPeriodProvided,
		ZeroPeriodLimitProvided,
		ZeroDripLimitProvided,
        ZeroDripAmountProvided,
        
        PeriodLimitReached,
        DripLimitReached,
	}
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        #[weight = T::DbWeight::get().reads_writes(1, 1) + 50_000]
        pub fn add_faucet(
            origin,
            faucet: T::AccountId,
            settings: FaucetSettings<T::BlockNumber, BalanceOf<T>>
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                Self::require_faucet_settings(&faucet).is_err(),
                Error::<T>::FaucetAlreadyAdded
            );

            ensure!(
                T::Currency::free_balance(&faucet) >= T::Currency::minimum_balance(),
                Error::<T>::NoFreeBalanceOnFaucet
            );

            Self::ensure_provided_valid_settings(&settings)?;
            SettingsByFaucet::<T>::insert(faucet.clone(), settings);
            Self::deposit_event(RawEvent::FaucetAdded(faucet));
            Ok(())
        }

        #[weight = T::DbWeight::get().reads_writes(1, 1) + 20_000]
        pub fn update_faucet(
            origin,
            faucet: T::AccountId,
            update: FaucetSettingsUpdate<T::BlockNumber, BalanceOf<T>>
        ) -> DispatchResult {
            ensure_root(origin)?;

            let has_updates =
                update.period.is_some() ||
                update.period_limit.is_some() ||
                update.drip_limit.is_some();

            ensure!(has_updates, Error::<T>::NoUpdatesProvided);

            let mut settings = Self::require_faucet_settings(&faucet)?;

            // `true` if there is at least one updated field.
            let mut should_update = false;

            if let Some(period) = update.period {
                Self::ensure_period_not_zero(period)?;

                if period != settings.period {
                    settings.period = period;
                    should_update = true;
                }
            }

            if let Some(period_limit) = update.period_limit {
                Self::ensure_period_limit_not_zero(period_limit)?;

                if period_limit != settings.period_limit {
                    ensure!(period_limit > Zero::zero(), Error::<T>::ZeroPeriodLimitProvided);
                    settings.period_limit = period_limit;
                    should_update = true;
                }
            }

            if let Some(drip_limit) = update.drip_limit {
                Self::ensure_drip_limit_not_zero(drip_limit)?;

                if drip_limit != settings.drip_limit {
                    settings.drip_limit = drip_limit;
                    should_update = true;
                }
            }

            return if should_update {
                SettingsByFaucet::<T>::insert(faucet.clone(), settings);
                Self::deposit_event(RawEvent::FaucetUpdated(faucet));
                Ok(())
            } else {
                Err(Error::<T>::NoUpdatesProvided.into())
            }
        }

        #[weight = T::DbWeight::get().reads_writes(0, 1) + 10_000 + 5_000 * faucets.len() as u64]
        pub fn remove_faucets(
            origin,
            faucets: Vec<T::AccountId>
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(faucets.len() != Zero::zero(), Error::<T>::NoFaucetsProvided);

            let unique_faucets = BTreeSet::from_iter(faucets.iter());
            for faucet in unique_faucets.iter() {
                SettingsByFaucet::<T>::remove(faucet);
            }

            Self::deposit_event(RawEvent::FaucetsRemoved(faucets));
            Ok(())
        }

        #[weight = (
            T::DbWeight::get().reads_writes(6, 4) + 50_000,
            Pays::No // TODO hm....
        )]
        pub fn drip(
            origin, // faucet account
            amount: BalanceOf<T>,
            recipient: T::AccountId
        ) -> DispatchResult {
            let faucet = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::ZeroDripAmountProvided);

            let settings = Self::require_faucet_settings(&faucet)?;
            ensure!(amount <= settings.drip_limit, Error::<T>::DripLimitReached);

            let mut faucet_drops_info = Self::faucet_drops_info(&faucet).unwrap_or_default();
            let current_block = <system::Module<T>>::block_number();

            if faucet_drops_info.next_period_at <= current_block {
                if let Some(period) = settings.period {
                    faucet_drops_info.next_period_at = current_block.saturating_add(period);
                    faucet_drops_info.dripped_in_current_period = Zero::zero();
                }
            }

            let amount_allowed = settings.period_limit.saturating_sub(faucet_drops_info.dripped_in_current_period);
            ensure!(amount <= amount_allowed, Error::<T>::PeriodLimitReached);

            T::Currency::transfer(
                &faucet,
                &recipient,
                amount,
                ExistenceRequirement::KeepAlive
            )?;

            faucet_drops_info.dripped_in_current_period = faucet_drops_info.dripped_in_current_period.saturating_add(amount);

            TotalFaucetDropsByAccount::<T>::mutate(&faucet, &recipient, |total| *total = total.saturating_add(amount));
            FaucetDropsInfo::<T>::insert(&faucet, faucet_drops_info);

            Self::deposit_event(RawEvent::TokensDropped(faucet, recipient, amount));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {

    pub fn require_faucet_settings(
        faucet: &T::AccountId
    ) -> Result<FaucetSettings<T::BlockNumber, BalanceOf<T>>, DispatchError> {
        Ok(Self::settings_by_faucet(faucet).ok_or(Error::<T>::FaucetNotFound)?)
    }

    fn ensure_period_not_zero(period_opt: Option<T::BlockNumber>) -> DispatchResult {
        if let Some(period) = period_opt {
            ensure!(period > Zero::zero(), Error::<T>::ZeroPeriodProvided);
        }
        Ok(())
    }

    fn ensure_period_limit_not_zero(period_limit: BalanceOf<T>) -> DispatchResult {
        ensure!(period_limit > Zero::zero(), Error::<T>::ZeroPeriodLimitProvided);
        Ok(())
    }

    fn ensure_drip_limit_not_zero(drip_limit: BalanceOf<T>) -> DispatchResult {
        ensure!(drip_limit > Zero::zero(), Error::<T>::ZeroDripLimitProvided);
        Ok(())
    }

    fn ensure_provided_valid_settings(settings: &FaucetSettings<T::BlockNumber, BalanceOf<T>>) -> DispatchResult {
        Self::ensure_period_not_zero(settings.period)?;
        Self::ensure_period_limit_not_zero(settings.period_limit)?;
        Self::ensure_drip_limit_not_zero(settings.drip_limit)
    }
}

impl<BlockNumber: Zero, Balance: Zero> Default for FaucetState<BlockNumber, Balance> {
    fn default() -> Self {
        Self {
            next_period_at: Zero::zero(),
            dripped_in_current_period: Zero::zero(),
        }
    }
}
