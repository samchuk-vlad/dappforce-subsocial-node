use crate::{Trait, WhoAndWhen, Encode, Decode};

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_runtime::SaturatedConversion;

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct WhoAndWhenSerializable<AccountId, BlockNumber> {
    pub account: AccountId,
    pub block: BlockNumber,
    pub time: u64,
}

impl<T: Trait> From<WhoAndWhen<T>> for WhoAndWhenSerializable<T::AccountId, T::BlockNumber> {
    fn from(from: WhoAndWhen<T>) -> Self {
        Self {
            account: from.account,
            block: from.block,
            time: from.time.saturated_into::<u64>(),
        }
    }
}