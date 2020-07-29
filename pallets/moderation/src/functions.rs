use crate::*;

impl Default for ReportFeedback {
    fn default() -> Self {
        Self::Confirm
    }
}

impl<T: Trait> Module<T> {
    pub fn require_report(report_id: ReportId) -> Result<Report<T>, DispatchError> {
        Ok(Self::report_by_id(report_id).ok_or(Error::<T>::ReportNotFound)?)
    }

    /// Get entity space_id if it exists.
    /// Content and Account has no scope, consider check with `if let Some`
    pub(crate) fn get_entity_scope(entity: &EntityId<T::AccountId>) -> Result<Option<SpaceId>, DispatchError> {
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
    pub fn new(
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