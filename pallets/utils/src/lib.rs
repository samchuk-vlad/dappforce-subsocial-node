#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage};
use sp_runtime::RuntimeDebug;
use pallet_timestamp;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct WhoAndWhen<T: Trait> {
  pub account: T::AccountId,
  pub block: T::BlockNumber,
  pub time: T::Moment,
}

impl <T:Trait> WhoAndWhen<T> {
  pub fn new(account: T::AccountId) -> Self {
    WhoAndWhen {
      account,
      block: <system::Module<T>>::block_number(),
      time: <pallet_timestamp::Module<T>>::now(),
    }
  }
}

pub trait Trait: system::Trait + pallet_timestamp::Trait {}

decl_storage! {
  trait Store for Module<T: Trait> as TemplateModule {

  }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}
