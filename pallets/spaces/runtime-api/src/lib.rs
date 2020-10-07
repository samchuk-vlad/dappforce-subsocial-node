#![cfg_attr(not(feature = "std"), no_std)]
// #![allow(clippy::too_many_arguments)]

use sp_std::vec::Vec;

use pallet_utils::SpaceId;
use pallet_spaces::{Trait as SpacesTrait, Space, WhoAndWhen1};

sp_api::decl_runtime_apis! {
    pub trait SpacesApi<T: SpacesTrait> {
        fn get_last_space_id() -> SpaceId;

        fn get_hidden_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId>;

        fn find_public_space_ids(offset: u64, limit: u64) -> Vec<SpaceId>;

        fn find_unlisted_space_ids(offset: u64, limit: u64) -> Vec<SpaceId>;

        // fn find_public_spaces(offset: u64, limit: u64) -> Vec<Space<T>>;

        fn find_struct() -> Vec<WhoAndWhen1<T>>;
    }
}
