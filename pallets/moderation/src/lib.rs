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
pub enum ReportFeedback {
    Confirm,
    Ignore,
}

impl Default for ReportFeedback {
    fn default() -> Self {
        Self::Confirm
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
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

    type AutoBlockConfirmations: Get<u16>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ModerationModule {
        NextReportId get(fn next_report_id): ReportId = 1;

        pub ReportById get(fn report_by_id):
            map hasher(twox_64_concat) ReportId => Option<Report<T>>;

        pub IsEntityReportedByAccount get(fn is_entity_reported_by_account):
            map hasher(twox_64_concat) (EntityId<T::AccountId>, T::AccountId) => bool;

        pub ReportIdsBySpaceId: map hasher(twox_64_concat) SpaceId => Vec<ReportId>;

        pub ReportIdsByEntityInSpace get(fn report_ids_by_entity_in_space): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) SpaceId
                => Vec<ReportId>;

        pub StatusesByEntityInSpace get(fn statuses_by_entity_in_space): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) SpaceId
                => Option<EntityStatus>;

        pub FeedbackByReportId get(fn feedback_by_report_id):
            map hasher(twox_64_concat) ReportId => Vec<(T::AccountId, ReportFeedback)>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId,
        EntityId = EntityId<<T as system::Trait>::AccountId>
    {
        EntityReported(AccountId, SpaceId, EntityId, ReportId),
        EntityBlocked(AccountId, SpaceId, EntityId),
        EntityAllowed(AccountId, SpaceId, EntityId),
        ReportConfirmed(AccountId, ReportId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The account has already made a report on this entity.
        AlreadyReported,
        /// Entity scope differs from the scope provided.
        EntityIsNotInScope,
        /// Entity was not found by its id.
        EntityNotFound,
        /// Entity scope provided doesn't exist.
        InvalidScope,
        /// Account has no permission to leave a feedback on reports.
        NoPermissionToManageReports,
        /// Report reason shouldn't be empty.
        ReasonIsEmpty,
        /// Report was not found by its id.
        ReportNotFound,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        const AutoBlockConfirmations: u16 = T::AutoBlockConfirmations::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Report any entity by any person with mandatory reason.
        /// `entity` scope and the `scope` provided mustn't differ
        #[weight = T::DbWeight::get().reads_writes(6, 5) + 10_000]
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
            ReportIdsBySpaceId::mutate(scope, |ids| ids.push(report_id));
            ReportIdsByEntityInSpace::<T>::mutate(&entity, scope, |ids| ids.push(report_id));
            NextReportId::mutate(|n| { *n += 1; });

            Self::deposit_event(RawEvent::EntityReported(who, scope, entity, report_id));
            Ok(())
        }

        /// Leave a feedback on the report either it's confirmation or ignore.
        /// `origin` - any permitted account (e.g. Space owner or moderator that's set via role)
        #[weight = 10_000]
        pub fn feedback_report(
            origin,
            report_id: ReportId,
            decision_opt: Option<ReportFeedback>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // todo(i): does `who` can delete report if he's a report creator?

            let report = Self::require_report(report_id)?;
            let space = Spaces::<T>::require_space(report.reported_within)?;
            Spaces::<T>::ensure_account_has_space_permission(
                who.clone(),
                &space,
                pallet_permissions::SpacePermission::ManageReports,
                Error::<T>::NoPermissionToManageReports.into(),
            )?;

            let feedback = decision_opt.unwrap_or_default();
            let mut feedbacks = FeedbackByReportId::<T>::take(report_id);
            feedbacks.push((who.clone(), feedback.clone()));
            // todo: rewrite old feedback

            let confirmations_total = feedbacks.iter()
                .filter(|(_, action)| *action == ReportFeedback::Confirm)
                .count();

            if confirmations_total >= T::AutoBlockConfirmations::get() as usize {
                // todo: block content automatically
            }

            FeedbackByReportId::<T>::insert(report_id, feedbacks);

            if feedback == ReportFeedback::Confirm {
                Self::deposit_event(RawEvent::ReportConfirmed(who, report_id));
            }
            Ok(())
        }

        /// Block any `entity` provided.
        /// `origin` - any permitted account (e.g. Space owner or moderator that's set via role)
        /// `forbid_content` - whether to block `Content` provided with entity.
        #[weight = 10_000]
        pub fn block_entity(
            origin,
            entity: EntityId<T::AccountId>,
            forbid_content: bool,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // todo:
            // 	- ensure whether entity exists `fn require_entity()`
            // 	- ensure whether `who` is `reported_within` space owner or permitted account

            // todo: blocking process
            // 	- EntityId::Content - add Blocked status on Space
            // 	- EntityId::Account - add Blocked status on Space
            // 	- EntityId::Space - add to block list and remove parent_id
            // 	- EntityId::Post - add to block list and move to Abbys

            Ok(())
        }

        #[weight = 10_000]
        pub fn unblock_entity(origin) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn require_report(report_id: ReportId) -> Result<Report<T>, DispatchError> {
        Ok(Self::report_by_id(report_id).ok_or(Error::<T>::ReportNotFound)?)
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

    #[allow(dead_code)]
    // fixme: do we need this?
    fn ensure_entity_exist(entity: &EntityId<T::AccountId>) -> DispatchResult {
        match entity {
            EntityId::Content(content) => {
                ensure!(!content.is_none(), Error::<T>::EntityNotFound);
                Ok(())
            },
            EntityId::Account(_) => Ok(()),
            EntityId::Space(space_id) => Spaces::<T>::ensure_space_exists(*space_id),
            EntityId::Post(post_id) => Posts::<T>::ensure_post_exists(*post_id),
        }.map_err(|_| Error::<T>::EntityNotFound.into())
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
