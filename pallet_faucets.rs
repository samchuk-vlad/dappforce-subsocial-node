//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_faucets::WeightInfo for WeightInfo {
	fn remove_faucets() -> Weight {
		(26_167_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
