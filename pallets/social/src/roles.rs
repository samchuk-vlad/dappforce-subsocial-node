use frame_support::{dispatch::{DispatchResult, DispatchError}};

use df_traits::{SpaceForRoles, SpaceForRolesProvider};
use pallet_permissions::SpacePermissionsContext;

use super::*;

impl<T: Trait> Module<T> {

    pub fn ensure_account_has_space_permission(
        account: T::AccountId,
        space: &Space<T>,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {

        let is_owner = space.is_owner(&account);
        let is_follower = Self::space_followed_by_account((account.clone(), space.id));

        let ctx = SpacePermissionsContext {
            space_id: space.id,
            is_space_owner: is_owner,
            is_space_follower: is_follower,
            space_perms: space.permissions.clone(),
        };

        T::Roles::ensure_account_has_space_permission(
            account,
            ctx,
            permission,
            error,
        )
    }
}

impl<T: Trait> SpaceForRolesProvider for Module<T> {
    type AccountId = T::AccountId;
    type SpaceId = SpaceId;

    fn get_space(id: Self::SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError> {
        let space: Space<T> = Module::space_by_id(id).ok_or(Error::<T>::SpaceNotFound)?;

        Ok(SpaceForRoles {
            owner: space.owner,
            permissions: space.permissions,
        })
    }

    fn is_space_follower(account: Self::AccountId, space_id: Self::SpaceId) -> bool {
        Module::<T>::space_followed_by_account((account, space_id))
    }
}
