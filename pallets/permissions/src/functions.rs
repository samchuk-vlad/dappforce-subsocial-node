use super::*;

use frame_support::{dispatch::{DispatchError}};

impl<T: Trait> Module<T> {
  pub fn is_actor_has_permission(actor: Actor<T::AccountId>, permission: SpacePermission) -> Result<bool, DispatchError> {
    Ok(true)
  }
}
