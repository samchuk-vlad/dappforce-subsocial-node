use spaces_runtime_api::SpaceSerializable;

use crate::*;

impl<T: Trait> Into<SpaceSerializable<T::AccountId, T::BlockNumber>> for Space<T> {
    fn into(self) -> SpaceSerializable<T::AccountId, T::BlockNumber> {
        SpaceSerializable {
            id: self.id,
            created: self.created.into(),
            updated: self.updated.map(|value| value.into()),
            owner: self.owner,
            parent_id: self.parent_id,
            handle: self.handle,
            content: self.content,
            hidden: self.hidden,
            posts_count: self.posts_count,
            hidden_posts_count: self.hidden_posts_count,
            followers_count: self.followers_count,
            score: self.score,
            permissions: self.permissions
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_last_space_id() -> SpaceId {
        Self::next_space_id().saturating_sub(1)
    }

    pub fn find_public_spaces(offset: u64, limit: u64) -> Vec<SpaceSerializable<T::AccountId, T::BlockNumber>> {
        let mut last_space_id = Self::next_space_id();
        last_space_id = last_space_id.saturating_sub(offset);

        let first_space_id: u64;
        first_space_id = last_space_id.saturating_sub(limit);

        let mut public_spaces = Vec::new();
        for space_id in first_space_id..=last_space_id {
            if let Some(space) = Self::require_space(space_id).ok() {
                if space.is_public() {
                    public_spaces.push(space.into());
                }
            }
        }

        public_spaces
    }

    pub fn find_unlisted_spaces(offset: u64, limit: u64) -> Vec<SpaceSerializable<T::AccountId, T::BlockNumber>> {
        let mut last_space_id = Self::next_space_id();

        last_space_id = last_space_id.saturating_sub(offset);

        let first_space_id: u64;
        first_space_id = last_space_id.saturating_sub(limit);

        let mut unlisted_spaces = Vec::new();

        for space_id in first_space_id..last_space_id {
            if let Some(space) = Self::require_space(space_id).ok() {
                if !space.is_public() {
                    unlisted_spaces.push(space.into());
                }
            }
        }

        unlisted_spaces
    }
}