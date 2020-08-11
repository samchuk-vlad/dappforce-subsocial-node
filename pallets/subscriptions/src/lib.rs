#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult,
	traits::Currency,
};
use frame_system::{self as system, ensure_signed};

use pallet_utils::{SpaceId, Content, WhoAndWhen};

/*#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;*/

pub type SubscriptionPlanId = u64;
pub type SubscriptionId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum SubscriptionPeriod<BlockNumber> {
	Custom(BlockNumber), // Currently not supported
	Daily,
	Weekly,
	Quarterly,
	Yearly,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SubscriptionPlan<T: Trait> {
	pub id: SubscriptionPlanId,
	pub created: WhoAndWhen<T>,
	pub updated: Option<WhoAndWhen<T>>,
	pub space_id: SpaceId, // Describes what space is this plan related to
	pub wallet: T::AccountId, // Describes where to transfer balance withdrawn from subscribers
	pub price: BalanceOf<T>,
	pub period: SubscriptionPeriod<T::BlockNumber>,
	pub content: Content,

	// ??? pub canceled: boolean,  // whether this plan was canceled by creator
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Subscription<T: Trait> {
	pub id: SubscriptionId,
	pub created: WhoAndWhen<T>,
	pub wallet: T::AccountId, // Describes where to withdraw balance for subscription from
	pub plan_id: SubscriptionPlanId,

	// ??? pub canceled: boolean, // whether this subscription was canceled by subscriber
}

type BalanceOf<T> = <<T as pallet_utils::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_utils::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SubscriptionsModule {
		// Plans:

		pub NextPlanId get(fn next_plan_id): SubscriptionPlanId = 1;

		pub PlanById get(fn plan_by_id):
			map hasher(twox_64_concat) SubscriptionPlanId => Option<SubscriptionPlan<T>>;

		pub PlanIdsBySpace get(fn plan_ids_by_space):
			map hasher(twox_64_concat) SpaceId => Vec<SubscriptionPlanId>;

		// Subscriptions:

		pub NextSubscriptionId get(fn next_subscription_id): SubscriptionId = 1;

		pub SubscriptionById get(fn subscription_by_id):
			map hasher(twox_64_concat) SubscriptionId => Option<Subscription<T>>;

		pub SubscriptionIdsByPatron get(fn subscription_ids_by_patron):
			map hasher(blake2_128_concat) T::AccountId => Vec<SubscriptionId>;

		pub SubscriptionIdsBySpace get(fn subscription_ids_by_space):
			map hasher(twox_64_concat) SpaceId => Vec<SubscriptionId>;

		// todo: this should be used by Scheduler to transfer funds from subscribers' wallets to creator's (space) wallet.
		pub SubscriptionIdsByPeriod get(fn subscription_ids_by_period):
			map hasher(twox_64_concat) SubscriptionPeriod<T::BlockNumber> => Vec<SubscriptionId>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId
	{
		SubscriptionPlanCreated(AccountId, SubscriptionPlanId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		type Error = Error<T>;

		// Initializing events
		fn deposit_event() = default;

		#[weight = 10_000]
		pub fn create_plan(
			origin,
			space_id: SpaceId,
			wallet: T::AccountId,
			price: BalanceOf<T>,
			period: T::BlockNumber,
			content: Content
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Ok(())
		}

		#[weight = 10_000]
		pub fn update_plan_wallet(
			origin,
			plan_id: SubscriptionPlanId,
			wallet: T::AccountId
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Ok(())
		}

		#[weight = 10_000]
		pub fn delete_plan(origin, plan_id: SubscriptionPlanId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Ok(())
		}

		#[weight = 10_000]
		pub fn subscribe(origin, plan_id: SubscriptionPlanId, wallet: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Ok(())
		}

		#[weight = 10_000]
		pub fn update_subscription_wallet(
			origin,
			subscription_id: SubscriptionId,
			wallet: T::AccountId
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Ok(())
		}

		#[weight = 10_000]
		pub fn unsubscribe(origin, plan_id: SubscriptionPlanId) -> DispatchResult {
			// todo(i): maybe we need here subscription_id, not plan_id?
			let _ = ensure_signed(origin)?;
			Ok(())
		}
	}
}
