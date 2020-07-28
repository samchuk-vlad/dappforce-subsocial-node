#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure,
	dispatch::{DispatchResult, DispatchError},
	traits::Get,
};
use frame_system::{self as system, ensure_signed};

use pallet_utils::{Content, WhoAndWhen, SpaceId, Module as Utils};
use pallet_posts::{PostId, Module as Posts};
use pallet_spaces::Module as Spaces;
/*
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
*/
pub type ReportId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityId<AccountId> {
	Content(Content),
	Account(AccountId),
	Space(SpaceId),
	Post(PostId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityStatus {
	Allowed,
	Blocked,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum ReportDecision {
	Confirm,
	Ignore,
}

impl Default for ReportDecision {
	fn default() -> Self {
		Self::Ignore
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
// todo(1): how to optimally get SpaceId from different entities?
pub struct Report<T: Trait> {
	id: ReportId,
	created: WhoAndWhen<T>,
	reported_entity: EntityId<T::AccountId>,
	reported_within: SpaceId,
	reason: Content,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
	+ pallet_posts::Trait
	+ pallet_spaces::Trait
	+ pallet_utils::Trait
{
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as ModerationModule {
		NextReportId get(fn next_report_id): ReportId = 1;

		pub ReportById get(fn report_by_id):
			map hasher(twox_64_concat) ReportId => Option<Report<T>>;

		pub IsEntityReportedByAccount get(fn is_entity_reported_by_account):
			map hasher(twox_64_concat) (EntityId<T::AccountId>, T::AccountId) => bool;

		pub ReportIdsByEntityInSpace get(fn report_ids_by_entity_in_space): double_map
			hasher(twox_64_concat) SpaceId,
			hasher(twox_64_concat) EntityId<T::AccountId>
				=> Vec<ReportId>;

		pub InSpaceEntityStatuses get(fn in_space_entity_statuses):
			map hasher(twox_64_concat) EntityId<T::AccountId> => Option<(SpaceId, EntityStatus)>;

		pub CoherenceByReportId get(fn coherence_by_report_id):
			map hasher(twox_64_concat) ReportId => Vec<(T::AccountId, ReportDecision)>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where
		AccountId = <T as system::Trait>::AccountId,
		EntityId = EntityId<<T as system::Trait>::AccountId>
	{
		EntityReported(AccountId, EntityId, ReportId),
		ReportConfirmed(AccountId, ReportId),
		ReportDenied(AccountId, ReportId),
		EntityBlocked(AccountId, EntityId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Entity was not found by it's id
		EntityNotFound,
		ReasonIsEmpty,
		AlreadyReported,
		InvalidScope,
		EntityIsNotInScope,
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

		/// Report any entity by any person with mandatory note.
		/// Could be confirmed by specifying it in `block_entity` dispatch.
		/// `reason` is a must
		#[weight = T::DbWeight::get().reads_writes(5, 4) + 10_000]
		pub fn report_entity(
			origin,
			scope: SpaceId,
			entity: EntityId<T::AccountId>,
			reason: Content
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!reason.is_none(), Error::<T>::ReasonIsEmpty);
			Utils::<T>::is_valid_content(reason.clone())?;

			ensure!(Spaces::<T>::require_space(scope).is_ok(), Error::<T>::InvalidScope);
			if let Some(entity_scope) = Self::get_entity_scope(&entity)? {
				ensure!(entity_scope == scope, Error::<T>::EntityIsNotInScope);
			}

			ensure!(!Self::is_entity_reported_by_account((&entity, &who)), Error::<T>::AlreadyReported);

			let report_id = Self::next_report_id();
			let new_report = Report::<T>::new(report_id, &who, entity.clone(), scope, reason);

			ReportById::<T>::insert(report_id, new_report);
			IsEntityReportedByAccount::<T>::insert((&entity, &who), true);
			ReportIdsByEntityInSpace::<T>::mutate(scope, &entity, |ids| ids.push(report_id));
			NextReportId::mutate(|n| { *n += 1; });

			Ok(())
		}

		/// Reject report by permitted account with or without a `reason`
		/// `origin` - any permitted account (e.g. Space owner or moderator that's set via role)
		/// `reason` could be `Content::None`
		#[weight = 10_000]
		pub fn deny_report(origin, report_id: ReportId, reason: Content) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// todo(4):
			// 	- verify `reason`
			// 	- ensure report exists
			// 	- ensure whether `who` is `reported_within` space owner or permitted account
			// 	- does `who` can deny report if he's report creator?
			// 	- remove or move report into another storage?

			Ok(())
		}

		/// Block any content provided with `entity`
		/// `origin` - any permitted account (e.g. Space owner or moderator that's set via role)
		/// `report_id` - if is Some, then the Report by specified id is treated as confirmed.
		#[weight = 10_000]
		pub fn block_entity(
			origin,
			entity: EntityId<T::AccountId>,
			report_id: Option<ReportId>
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// TODO: split on block/confirm

			// todo(5):
			// 	- ensure whether entity exists `fn require_entity()`
			// 	- ensure whether `who` is `reported_within` space owner or permitted account
			// 	- if `report_id` is Some, !check whether it exists
			// 	- if Report does exist, confirm a report (?); otherwise do nothing with it

			// 	todo(6): think on blocking process (Abbys or just hide?)

			Ok(())
		}

		// TODO: add unblock dispatch
	}
}

impl<T: Trait> Module<T> {
	#[allow(dead_code)]
	// fixme: do we need this?
	fn ensure_entity_exist(entity: &EntityId<T::AccountId>) -> DispatchResult {
		let result: DispatchResult = match entity {
			EntityId::Content(content) => {
				ensure!(!content.is_none(), Error::<T>::EntityNotFound);
				Ok(())
			},
			EntityId::Space(space_id) => Spaces::<T>::require_space(*space_id).map(|_| ()),
			EntityId::Post(post_id) => Posts::<T>::require_post(*post_id).map(|_| ()),
			EntityId::Account(_) => Ok(()),
		};

		result.map_err(|_| Error::<T>::EntityNotFound.into())
	}

	/// Get entity space_id if it exists.
	/// Content and Account has no scope, consider check with `if let Some`
	fn get_entity_scope(entity: &EntityId<T::AccountId>) -> Result<Option<SpaceId>, DispatchError> {
		match entity {
			EntityId::Content(_) => Ok(None),
			EntityId::Account(_) => Ok(None),
			EntityId::Space(space_id) => {
				let space = Spaces::<T>::require_space(*space_id)?;
				let root_space_id = space.try_get_parent()?;

				Ok(Some(root_space_id))
			},
			EntityId::Post(post_id) => {
				let post = Posts::<T>::require_post(*post_id)?;
				let space_id = post.get_space()?.id;

				Ok(Some(space_id))
			},
		}
	}
}

impl<T: Trait> Report<T> {
	fn new(
		id: ReportId,
		created_by: &T::AccountId,
		reported_entity: EntityId<T::AccountId>,
		scope: SpaceId,
		reason: Content
	) -> Self {
		Self {
			id,
			created: WhoAndWhen::<T>::new(created_by.clone()),
			reported_entity,
			reported_within: scope,
			reason
		}
	}
}
