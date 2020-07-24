#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::boxed_local)]

use codec::{Decode, Encode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{Dispatchable, Saturating, Zero};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    weights::GetDispatchInfo,
    dispatch::{DispatchError, DispatchResult, PostDispatchInfo},
    traits::{Currency, Get, ReservableCurrency, Imbalance, OnUnbalanced},
    Parameter,
};
use frame_system::{self as system, ensure_signed};

use pallet_utils::WhoAndWhen;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

// TODO define session key permissions

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SessionKey<T: Trait> {
    /// Who and when created this session key.
    pub created: WhoAndWhen<T>,

    /// The last time this session key was used or updated by its owner.
    pub updated: Option<WhoAndWhen<T>>,

    /// A block number when this session key should be expired.
    pub expires_at: T::BlockNumber,

    /// Max amount of tokens allowed to spend with this session key.
    pub limit: Option<BalanceOf<T>>,

    /// How much tokens this session key already spent.
    pub spent: BalanceOf<T>,

    // TODO allowed_actions: ...
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_utils::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The overarching call type.
    type Call: Parameter + Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo>
        + GetDispatchInfo + From<frame_system::Call<Self>>;

    /// The currency mechanism.
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    /// The maximum amount of session keys allowed for a single account.
    type MaxSessionKeysPerAccount: Get<u16>;
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>
    {
        SessionKeyAdded(/* owner */ AccountId, /* session key */ AccountId),
        SessionKeyRemoved(/* session key */ AccountId),
        AllSessionKeysRemoved(/* owner */ AccountId),
        /// A proxy was executed correctly, with the given result.
		ProxyExecuted(DispatchResult),
		Deposit(Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Session key details was not found by its account id.
        SessionKeyNotFound,
        /// Account already added as a session key.
        SessionKeyAlreadyAdded,
        /// There are too many session keys registered to this account.
        TooManySessionKeys,
        /// Time to live (TTL) of a session key cannot be zero.
        ZeroTimeToLive,
        /// Limit of a session key cannot be zero.
        ZeroLimit,
        /// Session key is expired.
        SessionKeyExpired,
        /// Reached the limit of tokens this session key can spend.
        SessionKeyLimitReached,
        /// Only a session key owner can manage their keys.
        NeitherSessionKeyOwnerNorExpired,
    }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as SessionKeysModule {

        /// Session key details by its account id (key).
        pub KeyDetails get(fn key_details):
            map hasher(blake2_128_concat)/* session key */ T::AccountId
            => Option<SessionKey<T>>;

        /// A binary-sorted list of all session keys owned by the account.
        pub KeysByOwner get(fn keys_by_owner):
            map hasher(twox_64_concat) /* primary owner */ T::AccountId
            => /* session keys */ Vec<T::AccountId>;

        TreasuryAccount build(|config| config.treasury_account.clone()): T::AccountId;
    }
    add_extra_genesis {
        config(treasury_account): T::AccountId;
        build(|config| {
			// Create Treasury account
			let _ = T::Currency::make_free_balance_be(
				&config.treasury_account,
				T::Currency::minimum_balance(),
			);
		});
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        const MaxSessionKeysPerAccount: u16 = T::MaxSessionKeysPerAccount::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        #[weight = 10_000]
        fn add_key(origin,
            key_account: T::AccountId,
            time_to_live: T::BlockNumber,
            limit: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(time_to_live > Zero::zero(), Error::<T>::ZeroTimeToLive);
            ensure!(limit != Some(Zero::zero()), Error::<T>::ZeroLimit);
            ensure!(!KeyDetails::<T>::contains_key(key_account.clone()), Error::<T>::SessionKeyAlreadyAdded);

            let mut keys = KeysByOwner::<T>::get(who.clone());
            ensure!(keys.len() < T::MaxSessionKeysPerAccount::get() as usize, Error::<T>::TooManySessionKeys);
            let i = keys.binary_search(&key_account).err().ok_or(Error::<T>::SessionKeyAlreadyAdded)?;
            keys.insert(i, key_account.clone());
            KeysByOwner::<T>::insert(&who, keys);

            let details = SessionKey::<T>::new(who.clone(), time_to_live, limit);
            KeyDetails::<T>::insert(key_account.clone(), details);

            Self::deposit_event(RawEvent::SessionKeyAdded(who, key_account));
            Ok(())
        }

        /// A key could be removed either the origin is an owner or key is expired.
        #[weight = 10_000]
        fn remove_key(origin, key_account: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let key = Self::require_key(key_account.clone())?;
            key.ensure_owner_or_expired(who.clone())?;

            KeyDetails::<T>::remove(key_account.clone());

            let mut keys = KeysByOwner::<T>::get(who.clone());
            let i = keys.binary_search(&key_account).ok().ok_or(Error::<T>::SessionKeyNotFound)?;
            keys.remove(i);
            KeysByOwner::<T>::insert(&who, keys);

            Self::deposit_event(RawEvent::SessionKeyRemoved(key_account));
            Ok(())
        }

        /// Unregister all session keys for the sender.
        #[weight = 10_000]
        fn remove_keys(origin) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let keys = KeysByOwner::<T>::take(&who);
            for key in keys {
                KeyDetails::<T>::remove(key);
            }

            Self::deposit_event(RawEvent::AllSessionKeysRemoved(who));
            Ok(())
        }

        /// `origin` is a session key
        #[weight = 10_000]
        fn proxy(origin, call: Box<<T as Trait>::Call>) -> DispatchResult {
            let key = ensure_signed(origin)?;

            let mut details = Self::require_key(key.clone())?;
            ensure!(!details.is_expired(), Error::<T>::SessionKeyExpired);

            // TODO we can delete a session key here?

            let real = details.owner();
            let mut can_spend: BalanceOf<T> = Zero::zero();
            let mut maybe_reserved: Option<BalanceOf<T>> = None;

            // TODO get limit from account settings

            if let Some(limit) = details.limit {
                can_spend = limit.saturating_sub(details.spent);
                ensure!(can_spend > Zero::zero(), Error::<T>::SessionKeyLimitReached);
                let cannot_spend = T::Currency::free_balance(&real).saturating_sub(can_spend);
                maybe_reserved = Some(cannot_spend);
                T::Currency::reserve(&real, cannot_spend)?;
            }

            // TODO check that this call is among allowed calls per this account/session key.

            let e = call.dispatch(system::RawOrigin::Signed(real.clone()).into());
            Self::deposit_event(RawEvent::ProxyExecuted(e.map(|_| ()).map_err(|e| e.error)));

            if let Some(reserved) = maybe_reserved {
                let spent_on_call = can_spend.saturating_sub(T::Currency::free_balance(&real));
                T::Currency::unreserve(&real, reserved);
                if spent_on_call > Zero::zero() {

                    // TODO update 'spent' even if nothing was reserved.

                    details.spent = details.spent.saturating_add(spent_on_call);
                    details.updated = Some(WhoAndWhen::<T>::new(key.clone()));
                    KeyDetails::<T>::insert(key, details);
                }
            }

            Ok(())
        }

        // TODO write a scheduler to remove expired session keys.
    }
}

impl<T: Trait> SessionKey<T> {
    pub fn new(
        created_by: T::AccountId,
        time_to_live: T::BlockNumber,
        limit: Option<BalanceOf<T>>,
    ) -> Self {
        SessionKey::<T> {
            created: WhoAndWhen::new(created_by),
            updated: None,
            expires_at: time_to_live + <system::Module<T>>::block_number(),
            limit,
            spent: Zero::zero(),
        }
    }

    pub fn owner(&self) -> T::AccountId {
        self.created.account.clone()
    }

    pub fn is_owner(&self, maybe_owner: &T::AccountId) -> bool {
        self.owner() == *maybe_owner
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= <system::Module<T>>::block_number()
    }

    pub fn ensure_owner_or_expired(&self, maybe_owner: T::AccountId) -> DispatchResult {
        ensure!(
            self.is_owner(&maybe_owner) || self.is_expired(),
            Error::<T>::NeitherSessionKeyOwnerNorExpired
        );
        Ok(())
    }
}

impl<T: Trait> Module<T> {
    /// Get `SessionKey` details by `key_account` from the storage
    /// or return `SessionKeyNotFound` error.
    pub fn require_key(key_account: T::AccountId) -> Result<SessionKey<T>, DispatchError> {
        Ok(Self::key_details(key_account).ok_or(Error::<T>::SessionKeyNotFound)?)
    }
}

impl<T: Trait> OnUnbalanced<NegativeImbalanceOf<T>> for Module<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
        let numeric_amount = amount.peek();
        let treasury_account = TreasuryAccount::<T>::get();

        // Must resolve into existing but better to be safe.
        let _ = T::Currency::resolve_creating(&treasury_account, amount);

        Self::deposit_event(RawEvent::Deposit(numeric_amount));
    }
}
