use crate::*;

use spaces_runtime_api::SpaceInfo;

impl<T: Trait> Module<T> {
    pub fn find_struct(space_id: SpaceId) -> SpaceInfo<T::AccountId> {
        let space_opt: Option<Space<T>> = Self::require_space(space_id).ok();
        return if let Some(space) = space_opt {
            SpaceInfo {
                id: space.id,
                owner: space.owner,
                parent_id: space.parent_id,
                handle: space.handle,
                hidden: space.hidden,
                posts_count: space.posts_count,
                hidden_posts_count: space.hidden_posts_count,
                followers_count: space.followers_count,
                score: space.score,
                permissions: space.permissions
            }
        } else {
            SpaceInfo::default()
        };
    }
}