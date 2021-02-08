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
use pallet_utils::{Trait as UtilsTrait};
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

// TODO rename
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetSettings<T: Trait> {
    // Settings
    pub is_active: bool,
    pub period: T::BlockNumber,
    pub period_limit: BalanceOf<T>,
    pub drip_limit: BalanceOf<T>,

    // State
    pub next_period_at: T::BlockNumber,
    pub dripped_in_current_period: BalanceOf<T>,
}

// TODO rename
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetSettingsUpdate<T: Trait> {
    pub is_active: Option<bool>,
    pub period: Option<T::BlockNumber>,
    pub period_limit: Option<BalanceOf<T>>,
    pub drip_limit: Option<BalanceOf<T>>,
}

type BalanceOf<T> = <<T as UtilsTrait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_utils::Trait + sp_std::fmt::Debug {

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as FaucetsModule {

        /// Get a faucet data by its account id.
		pub FaucetByAccount get(fn faucet_by_account):
			map hasher(twox_64_concat) T::AccountId // Faucet account
			=> Option<FaucetSettings<T>>;
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
		Dripped(
			AccountId, // Faucet account
			AccountId, // Recipient account
			Balance    // Amount dripped
		),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		FaucetNotFound,
		FaucetAlreadyAdded,
        NoFreeBalanceOnFaucet,
        NoFaucetsProvided,
        NoUpdatesProvided,
        NothingToUpdate,
        FaucetDisabled,
        NotFaucetOwner,

		ZeroPeriodProvided,
		ZeroPeriodLimitProvided,
		ZeroDripLimitProvided,
        ZeroDripAmountProvided,
        
        PeriodLimitReached,
        DripLimitReached,
	}
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        // TODO review read/writes
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 50_000]
        pub fn add_faucet(
            origin,
            faucet: T::AccountId,
            period: T::BlockNumber,
            period_limit: BalanceOf<T>,
            drip_limit: BalanceOf<T>,
        ) -> DispatchResult {

            ensure_root(origin.clone())?;

            Self::ensure_period_not_zero(period)?;
            Self::ensure_period_limit_not_zero(period_limit)?;
            Self::ensure_drip_limit_not_zero(drip_limit)?;

            ensure!(
                Self::faucet_by_account(&faucet).is_none(),
                Error::<T>::FaucetAlreadyAdded
            );

            ensure!(
                <T as UtilsTrait>::Currency::free_balance(&faucet) >=
                <T as UtilsTrait>::Currency::minimum_balance(),
                Error::<T>::NoFreeBalanceOnFaucet
            );

            let new_faucet = FaucetSettings::<T>::new(
                period,
                period_limit,
                drip_limit
            );

            FaucetByAccount::<T>::insert(faucet.clone(), new_faucet);
            Self::deposit_event(RawEvent::FaucetAdded(faucet));
            Ok(())
        }

        // TODO review read/writes
        #[weight = T::DbWeight::get().reads_writes(1, 1) + 20_000]
        pub fn update_faucet(
            origin,
            faucet: T::AccountId,
            update: FaucetSettingsUpdate<T>
        ) -> DispatchResult {
            ensure_root(origin)?;

            let has_updates =
                update.is_active.is_some() ||
                update.period.is_some() ||
                update.period_limit.is_some() ||
                update.drip_limit.is_some();

            ensure!(has_updates, Error::<T>::NoUpdatesProvided);

            let mut settings = Self::require_faucet(&faucet)?;

            // `true` if there is at least one updated field.
            let mut should_update = false;

            if let Some(is_active) = update.is_active {
                if is_active != settings.is_active {
                    settings.is_active = is_active;
                    should_update = true;
                }
            }

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

            ensure!(should_update, Error::<T>::NothingToUpdate);

            FaucetByAccount::<T>::insert(faucet.clone(), settings);
            Self::deposit_event(RawEvent::FaucetUpdated(faucet));
            Ok(())
        }

        // TODO review read/writes
        #[weight = T::DbWeight::get().reads_writes(0, 1) + 10_000 + 5_000 * faucets.len() as u64]
        pub fn remove_faucets(
            origin,
            faucets: Vec<T::AccountId>
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(faucets.len() != Zero::zero(), Error::<T>::NoFaucetsProvided);

            let unique_faucets = BTreeSet::from_iter(faucets.iter());
            for faucet in unique_faucets.iter() {
                FaucetByAccount::<T>::remove(faucet);
            }

            Self::deposit_event(RawEvent::FaucetsRemoved(faucets));
            Ok(())
        }

        // TODO review read/writes
        #[weight = (
            T::DbWeight::get().reads_writes(6, 4) + 50_000,
            Pays::No
        )]
        pub fn drip(
            origin, // faucet account
            recipient: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let faucet = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::ZeroDripAmountProvided);

            let mut settings = Self::require_faucet(&faucet)?;
            ensure!(settings.is_active, Error::<T>::FaucetDisabled);
            ensure!(amount <= settings.drip_limit, Error::<T>::DripLimitReached);

            let current_block = <system::Module<T>>::block_number();

            if settings.next_period_at <= current_block {
                settings.next_period_at = current_block.saturating_add(settings.period);
                settings.dripped_in_current_period = Zero::zero();
            }

            let amount_allowed = settings.period_limit
                .saturating_sub(settings.dripped_in_current_period);

            ensure!(amount <= amount_allowed, Error::<T>::PeriodLimitReached);

            <T as UtilsTrait>::Currency::transfer(
                &faucet,
                &recipient,
                amount,
                ExistenceRequirement::KeepAlive
            )?;

            settings.dripped_in_current_period = amount
                .saturating_add(settings.dripped_in_current_period);

            FaucetByAccount::<T>::insert(&faucet, settings);

            Self::deposit_event(RawEvent::Dripped(faucet, recipient, amount));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {

    pub fn require_faucet(faucet: &T::AccountId) -> Result<FaucetSettings<T>, DispatchError> {
        Ok(Self::faucet_by_account(faucet).ok_or(Error::<T>::FaucetNotFound)?)
    }

    fn ensure_period_not_zero(period: T::BlockNumber) -> DispatchResult {
        ensure!(period > Zero::zero(), Error::<T>::ZeroPeriodProvided);
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
}

impl<T: Trait> FaucetSettings<T> {

    pub fn new(
        period: T::BlockNumber,
        period_limit: BalanceOf<T>,
        drip_limit: BalanceOf<T>,
    ) -> Self {
        Self {
            is_active: true,
            period,
            period_limit,
            drip_limit,

            next_period_at: Zero::zero(),
            dripped_in_current_period: Zero::zero(),
        }
    }
}
