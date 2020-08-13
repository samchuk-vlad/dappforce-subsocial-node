use crate::*;

use frame_support::dispatch::DispatchError;

impl<T: Trait> Module<T> {
    pub fn require_plan(
        plan_id: SubscriptionPlanId
    ) -> Result<SubscriptionPlan<T>, DispatchError> {
        Ok(Self::plan_by_id(plan_id).ok_or(Error::<T>::SubscriptionPlanNotFound)?)
    }

    pub fn require_subscription(subscription_id: SubscriptionId) -> Result<Subscription<T>, DispatchError> {
        Ok(Self::subscription_by_id(subscription_id).ok_or(Error::<T>::SubscriptionNotFound)?)
    }

    pub fn ensure_subscriptions_manager(account: T::AccountId, space: &Space<T>) -> DispatchResult {
        Spaces::<T>::ensure_account_has_space_permission(
            account,
            space,
            SpacePermission::ManageSubscriptionPlans,
            Error::<T>::NoPermissionToUpdateSubscriptionPlan.into()
        )
    }
}

impl<T: Trait> SubscriptionPlan<T> {
    pub fn new(
        id: SubscriptionPlanId,
        created_by: T::AccountId,
        space_id: SpaceId,
        wallet: Option<T::AccountId>,
        price: BalanceOf<T>,
        period: SubscriptionPeriod<T::BlockNumber>,
        content: Content
    ) -> Self {
        Self {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            space_id,
            wallet,
            price,
            period,
            content,
            is_active: true
        }
    }
}

impl<T: Trait> Subscription<T> {
    pub fn new(
        id: SubscriptionId,
        created_by: T::AccountId,
        wallet: Option<T::AccountId>,
        plan_id: SubscriptionPlanId
    ) -> Self {
        Self {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            wallet,
            plan_id,
            is_active: true
        }
    }

    pub fn ensure_subscriber(&self, who: &T::AccountId) -> DispatchResult {
        ensure!(&self.created.account == who, Error::<T>::NotSubscriber);
        Ok(())
    }
}