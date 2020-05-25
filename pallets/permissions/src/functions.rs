use super::*;

use frame_support::{dispatch::{DispatchError}};

impl<T: Trait> Module<T> {
  pub fn is_user_has_permission(user: User<T::AccountId>, permission: SpacePermission) -> Result<bool, DispatchError> {
    Ok(true)
  }
}
