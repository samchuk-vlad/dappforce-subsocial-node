use crate::*;

use spaces_runtime_api::SpaceSerializable;

impl<T: Trait> Module<T> {
    pub fn find_struct(space_id: SpaceId) -> SpaceSerializable<T::AccountId, T::BlockNumber> {
        let space_opt: Option<Space<T>> = Self::require_space(space_id).ok();
        return if let Some(space) = space_opt {
            SpaceSerializable {
                id: space.id,
                created: space.created.into(),
                updated: space.updated.map(|value| value.into()),
                owner: space.owner,
                parent_id: space.parent_id,
                handle: space.handle,
                content: space.content,
                hidden: space.hidden,
                posts_count: space.posts_count,
                hidden_posts_count: space.hidden_posts_count,
                followers_count: space.followers_count,
                score: space.score,
                permissions: space.permissions
            }
        } else {
            SpaceSerializable::default()
        };
    }
}