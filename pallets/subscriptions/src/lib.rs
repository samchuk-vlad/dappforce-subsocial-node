#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure,
	dispatch::{DispatchResult, DispatchError},
	traits::{Get, Currency},
};
use frame_system::{self as system, ensure_signed};

use pallet_spaces::Module as Spaces;
use pallet_utils::{Module as Utils, SpaceId, Content, WhoAndWhen};

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
	pub price: BalanceOf<T>,
	pub period: SubscriptionPeriod<T::BlockNumber>,
	pub content: Content,

	// ??? pub canceled: boolean,  // whether this plan was canceled by creator
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Subscription<T: Trait> {
	pub id: SubscriptionId,
	pub created: WhoAndWhen<T>,
	pub plan_id: SubscriptionPlanId,

	// ??? pub canceled: boolean, // whether this subscription was canceled by subscriber
}

type BalanceOf<T> = <<T as pallet_utils::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait:
	system::Trait
	+ pallet_utils::Trait
	+ pallet_spaces::Trait
{
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

		// Wallets

		// Where to transfer balance withdrawn from subscribers
		pub RecipientWallet get(fn recipient_wallet):
			map hasher(twox_64_concat) (SpaceId, SubscriptionPlanId) => Option<T::AccountId>;

		// From where to withdraw subscribers donation
		pub PatronWallet get(fn patron_wallet):
			map hasher(twox_64_concat) (T::AccountId, SubscriptionId) => Option<T::AccountId>;
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
		NotAllowedToUpdatePlanWallet,
		NotAllowedToUpdateSubscriptionWallet,
		NothingToUpdate,
		PriceLowerExistencialDeposit,
		SubscriptionNotFound,
		SubscriptionPlanNotFound,
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		type Error = Error<T>;

		// Initializing events
		fn deposit_event() = default;

		#[weight = T::DbWeight::get().reads_writes(3, 4) + 25_000]
		pub fn create_plan(
			origin,
			space_id: SpaceId,
			custom_wallet: Option<T::AccountId>,
			price: BalanceOf<T>,
			period: SubscriptionPeriod<T::BlockNumber>,
			content: Content
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Utils::<T>::is_valid_content(content.clone())?;

			ensure!(
				price >= <T as pallet_utils::Trait>::Currency::minimum_balance(),
				Error::<T>::PriceLowerExistencialDeposit
			);

			let space = Spaces::<T>::require_space(space_id)?;
			space.ensure_space_owner(sender.clone())?;

			// todo:
			// 	- maybe add permission to create subscription plans?
			// 	- add max subscription plans per space?

			let plan_id = Self::next_plan_id();
			let subscription_plan = SubscriptionPlan::<T>::new(
				plan_id,
				sender.clone(),
				space_id,
				price,
				period,
				content
			);

			PlanById::<T>::insert(plan_id, subscription_plan);
			PlanIdsBySpace::mutate(space_id, |ids| ids.push(plan_id));
			NextPlanId::mutate(|x| { *x += 1 });

			RecipientWallet::<T>::insert((space_id, plan_id), custom_wallet.unwrap_or(sender));

			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(2, 1) + 10_000]
		pub fn update_plan_wallet(
			origin,
			plan_id: SubscriptionPlanId,
			custom_wallet: Option<T::AccountId>
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let plan = Self::require_subscription_plan(plan_id)?;
			plan.ensure_has_permission_to_update(&sender)?;

			// todo(i): do we need this check? Implementing this adds +1 reads in worst case.
			// ensure!(
			// 	Self::recipient_wallet((plan.space_id, plan_id)) != custom_wallet,
			// 	Error::<T>::NothingToUpdate
			// );

			RecipientWallet::<T>::insert((plan.space_id, plan_id), custom_wallet.unwrap_or(sender));

			Ok(())
		}

		#[weight = 10_000]
		pub fn delete_plan(origin, plan_id: SubscriptionPlanId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Ok(())
		}

		#[weight = 10_000]
		pub fn subscribe(
			origin,
			plan_id: SubscriptionPlanId,
			custom_wallet: Option<T::AccountId>
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Ok(())
		}

		#[weight = T::DbWeight::get().reads_writes(1, 1) + 10_000]
		pub fn update_subscription_wallet(
			origin,
			subscription_id: SubscriptionId,
			custom_wallet: Option<T::AccountId>
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let subscription = Self::require_subscription(subscription_id)?;
			subscription.ensure_has_permission_to_update(&sender)?;

			// todo(i): do we need this check? Implementing this adds +1 reads in worst case.
			// ensure!(
			// 	Self::patron_wallet((sender.clone(), subscription_id)) != custom_wallet,
			// 	Error::<T>::NothingToUpdate
			// );

			PatronWallet::<T>::insert((sender.clone(), subscription_id), custom_wallet.unwrap_or(sender));

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

impl<T: Trait> Module<T> {
	pub fn require_subscription_plan(
		plan_id: SubscriptionPlanId
	) -> Result<SubscriptionPlan<T>, DispatchError> {
		Ok(Self::plan_by_id(plan_id).ok_or(Error::<T>::SubscriptionPlanNotFound)?)
	}

	pub fn require_subscription(subscription_id: SubscriptionId) -> Result<Subscription<T>, DispatchError> {
		Ok(Self::subscription_by_id(subscription_id).ok_or(Error::<T>::SubscriptionNotFound)?)
	}
}

impl<T: Trait> SubscriptionPlan<T> {
	fn new(
		id: SubscriptionPlanId,
		created_by: T::AccountId,
		space_id: SpaceId,
		price: BalanceOf<T>,
		period: SubscriptionPeriod<T::BlockNumber>,
		content: Content
	) -> Self {
		Self {
			id,
			created: WhoAndWhen::<T>::new(created_by),
			updated: None,
			space_id,
			price,
			period,
			content
		}
	}

	fn ensure_has_permission_to_update(&self, who: &T::AccountId) -> DispatchResult {
		Spaces::<T>::require_space(self.space_id).and_then(|space| {
			ensure!(
				&self.created.account == who && space.is_owner(who),
				Error::<T>::NotAllowedToUpdatePlanWallet
			);
			Ok(())
		})
	}
}

impl<T: Trait> Subscription<T> {
	fn new(id: SubscriptionId, created_by: T::AccountId, plan_id: SubscriptionPlanId) -> Self {
		Self {
			id,
			created: WhoAndWhen::<T>::new(created_by),
			plan_id
		}
	}

	fn ensure_has_permission_to_update(&self, who: &T::AccountId) -> DispatchResult {
		ensure!(&self.created.account == who, Error::<T>::NotAllowedToUpdateSubscriptionWallet);
		Ok(())
	}
}
