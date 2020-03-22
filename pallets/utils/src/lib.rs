#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct WhoAndWhen<T: system::Trait + pallet_timestamp::Trait> {
  pub account: T::AccountId,
  pub block: T::BlockNumber,
  pub time: T::Moment,
}

impl <T:system::Trait + pallet_timestamp::Trait> WhoAndWhen<T> {
  pub fn new(account: T::AccountId) -> Self {
    WhoAndWhen {
      account,
      block: <system::Module<T>>::block_number(),
      time: <pallet_timestamp::Module<T>>::now(),
    }
  }
}
