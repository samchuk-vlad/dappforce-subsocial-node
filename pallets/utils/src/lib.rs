#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use frame_support::{decl_module, decl_error, ensure, traits::Get, dispatch::DispatchResult};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct WhoAndWhen<T: Trait> {
  pub account: T::AccountId,
  pub block: T::BlockNumber,
  pub time: T::Moment,
}

impl <T: Trait> WhoAndWhen<T> {
  pub fn new(account: T::AccountId) -> Self {
    WhoAndWhen {
      account,
      block: <system::Module<T>>::block_number(),
      time: <pallet_timestamp::Module<T>>::now(),
    }
  }
}

pub trait Trait: system::Trait + pallet_timestamp::Trait {
  /// The length in bytes of IPFS hash
  type IpfsHashLen: Get<u32>;
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    /// The length in bytes of IPFS hash
    const IpfsHashLen: u32 = T::IpfsHashLen::get();
  }
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// IPFS-hash is not correct
    IpfsIsIncorrect,
  }
}

impl<T: Trait> Module<T> {
  pub fn is_ipfs_hash_valid(ipfs_hash: Vec<u8>) -> DispatchResult {
    ensure!(ipfs_hash.len() == T::IpfsHashLen::get() as usize, Error::<T>::IpfsIsIncorrect);
    Ok(())
  }
}
