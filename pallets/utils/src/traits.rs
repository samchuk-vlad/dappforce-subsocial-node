use frame_support::dispatch::{DispatchResult, DispatchError};

pub trait SpaceSupported {
  type SpaceId;

  type SpaceOwner;

  type EveryonePermissions;

  type FollowerPermissions;


  fn get_space_owner_by_space_id(id: Self::SpaceId) -> Result<Self::SpaceOwner, DispatchError>;

  fn get_everyone_permissions_by_space_id(id: Self::SpaceId) -> Result<Self::EveryonePermissions, DispatchError>;

  fn get_follower_permissions_by_space_id(id: Self::SpaceId) -> Result<Self::FollowerPermissions, DispatchError>;
}
