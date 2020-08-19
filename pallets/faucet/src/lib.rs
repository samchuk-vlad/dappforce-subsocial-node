#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{
	prelude::*,
	iter::FromIterator,
	collections::btree_set::BTreeSet,
};

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure,
	dispatch::{DispatchResult, DispatchError},
	traits::{Get, Currency, ExistenceRequirement},
	weights::Pays,
};
use frame_system::{self as system, ensure_signed, ensure_root};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

type DripId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetSettings<BlockNumber, Balance> {
	period: Option<BlockNumber>,
	limit: Balance
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct FaucetSettingsUpdate<BlockNumber, Balance> {
	period: Option<Option<BlockNumber>>,
	limit: Option<Balance>
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Drop<T: Trait> {
	id: DripId,
	first_drop: T::BlockNumber,
	dropped_amount: BalanceOf<T>
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
		NextDripId get(fn next_drip_id): DripId = 1;

		pub DropById get(fn drop_by_id):
			map hasher(twox_64_concat) DripId => Option<Drop<T>>;

		pub DripIdByAccount get(fn drip_id_by_account):
			map hasher(twox_64_concat) T::AccountId => Option<DripId>;

		pub DripIdByAlias get(fn drip_id_by_alias):
			map hasher(blake2_128_concat) Vec<u8> => Option<DripId>;

		pub FaucetSettingsByAccount get(fn faucet_settings_by_account):
			map hasher(twox_64_concat) T::AccountId => Option<FaucetSettings<T::BlockNumber, BalanceOf<T>>>;
	}
}

decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>
	{
		NewFaucetCreated,
		FaucetUpdated,
		FaucetsRemoved,
		FaucetDropped(AccountId, Balance),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		FaucetAlreadyExists,
		FaucetLimitReached,
		FaucetNotFound,
		NoFreeBalanceOnAccount,
		NothingToUpdate,
		ZeroAmount,
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

		#[weight = T::DbWeight::get().reads_writes(1, 1) + 10_000]
		pub fn add_faucet(
			origin,
			faucet_account: T::AccountId,
			settings: FaucetSettings<T::BlockNumber, BalanceOf<T>>
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(Self::require_faucet(&faucet_account).is_ok(), Error::<T>::FaucetAlreadyExists);
			ensure!(
				T::Currency::free_balance(&faucet_account) >= T::Currency::minimum_balance(),
				Error::<T>::NoFreeBalanceOnAccount
			);

			FaucetSettingsByAccount::<T>::insert(faucet_account, settings);

			Self::deposit_event(RawEvent::NewFaucetCreated);
			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(1, 1) + 20_000]
		pub fn update_faucet(
			origin,
			faucet_account: T::AccountId,
			settings_update: FaucetSettingsUpdate<T::BlockNumber, BalanceOf<T>>
		) -> DispatchResult {
			ensure_root(origin)?;

			let has_updates =
				settings_update.period.is_some() ||
				settings_update.limit.is_some();

			ensure!(has_updates, Error::<T>::NothingToUpdate);
			let mut faucet_settings = Self::require_faucet(&faucet_account)?;
			let mut is_update_applied = false;

			if let Some(period) = settings_update.period {
				if period != faucet_settings.period {
					faucet_settings.period = period;
					is_update_applied = true;
				}
			}

			if let Some(limit) = settings_update.limit {
				if limit != faucet_settings.limit {
					faucet_settings.limit = limit;
					is_update_applied = true;
				}
			}

			if is_update_applied {
				FaucetSettingsByAccount::<T>::insert(faucet_account, faucet_settings);
				Self::deposit_event(RawEvent::FaucetUpdated);
			}
			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(0, 1) + 10_000
			+ 5_000 * faucet_accounts.len() as u64
		]
		pub fn remove_faucets(
			origin,
			faucet_accounts: Vec<T::AccountId>
		) -> DispatchResult {
			ensure_root(origin.clone())?;
			let root_key = ensure_signed(origin)?;

			let accounts_set = BTreeSet::from_iter(faucet_accounts.iter());
			for account in accounts_set.iter() {
				if Self::require_faucet(account).is_ok() {
					T::Currency::transfer(
						account,
						&root_key,
						T::Currency::free_balance(account),
						ExistenceRequirement::AllowDeath
					)?;
				}

				FaucetSettingsByAccount::<T>::remove(account);
			}

			Self::deposit_event(RawEvent::FaucetsRemoved);
			Ok(())
		}

		#[weight = (
			T::DbWeight::get().reads_writes(6, 4) + 50_000,
			Pays::No
		)]
		pub fn drip(
			origin,
			amount: BalanceOf<T>,
			faucet_account: T::AccountId,
			destination: T::AccountId,
			alias: Vec<u8>
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(amount != Zero::zero(), Error::<T>::ZeroAmount);

			let faucet_settings = Self::require_faucet(&faucet_account)?;
			let drop_opt = Self::drip_id_by_account(&faucet_account)
				.ok_or_else(|| Self::drip_id_by_alias(&alias))
				.ok()
				.and_then(Self::drop_by_id);

			let mut is_new_drop = false;
			let mut drop = drop_opt.unwrap_or_else(|| {
				let drip_id = Self::next_drip_id();
				is_new_drop = true;
				Drop::<T>::new(drip_id)
			});

			if !is_new_drop {
				let now = <system::Module<T>>::block_number();
				let past = now.saturating_sub(faucet_settings.period.unwrap_or_else(Zero::zero));

				if past >= drop.first_drop {
					drop.first_drop = now;
					if faucet_settings.period.is_some() {
						drop.dropped_amount = Zero::zero();
					}
				}
			}

			let amount_allowed = faucet_settings.limit.saturating_sub(drop.dropped_amount);
			ensure!(amount_allowed >= amount, Error::<T>::FaucetLimitReached);

			T::Currency::transfer(
				&faucet_account,
				&destination,
				amount,
				ExistenceRequirement::KeepAlive
			)?;

			drop.dropped_amount = drop.dropped_amount.saturating_add(amount);

			DropById::<T>::insert(drop.id, drop.clone());
			if is_new_drop {
				DripIdByAccount::<T>::insert(&destination, drop.id);
				DripIdByAlias::insert(alias, drop.id);
				NextDripId::mutate(|x| *x += 1);
			}

			Self::deposit_event(RawEvent::FaucetDropped(destination, amount));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn require_faucet(
		account: &T::AccountId
	) -> Result<FaucetSettings<T::BlockNumber, BalanceOf<T>>, DispatchError> {
		Ok(Self::faucet_settings_by_account(account).ok_or(Error::<T>::FaucetNotFound)?)
	}
}

impl<T: Trait> Drop<T> {
	pub fn new(id: DripId) -> Self {
		Self {
			id,
			first_drop: <system::Module<T>>::block_number(),
			dropped_amount: Zero::zero()
		}
	}
}
