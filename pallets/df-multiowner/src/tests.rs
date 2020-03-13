#![cfg(test)]

pub use super::*;

use sp_core::H256;
use frame_support::{impl_outer_origin, assert_ok, assert_noop, parameter_types, weights::Weight, dispatch::DispatchResult};
use sp_runtime::{
  traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};

impl_outer_origin! {
  pub enum Origin for Test {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const MaximumBlockWeight: Weight = 1024;
  pub const MaximumBlockLength: u32 = 2 * 1024;
  pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl system::Trait for Test {
  type Origin = Origin;
  type Call = ();
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = u64;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type Event = ();
  type BlockHashCount = BlockHashCount;
  type MaximumBlockWeight = MaximumBlockWeight;
  type MaximumBlockLength = MaximumBlockLength;
  type AvailableBlockRatio = AvailableBlockRatio;
  type Version = ();
  type ModuleToIndex = ();
}

parameter_types! {
  pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Trait for Test {
  type Moment = u64;
  type OnTimestampSet = ();
  type MinimumPeriod = MinimumPeriod;
}

impl Trait for Test {
  type Event = ();
}

type MultiOwnership = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
fn new_test_ext() -> sp_io::TestExternalities {
  system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

type AccountId = u64;

const ACCOUNT1 : AccountId = 1;
const ACCOUNT2 : AccountId = 2;
const ACCOUNT3 : AccountId = 3;

/*fn blog_update(writers: Option<Vec<AccountId>>, slug: Option<Vec<u8>>, ipfs_hash: Option<Vec<u8>>) -> BlogUpdate<u64> {
  BlogUpdate {
    writers,
    slug,
    ipfs_hash
  }
}*/

fn _create_default_space_owners() -> DispatchResult {
  _create_space_owners(None, None, None, None)
}

fn _create_space_owners(
  origin: Option<Origin>,
  space_id: Option<SpaceId>,
  owners: Option<Vec<AccountId>>,
  threshold: Option<u16>
) -> DispatchResult {

  MultiOwnership::create_space_owners(
    origin.unwrap_or(Origin::signed(ACCOUNT1)),
    space_id.unwrap_or(1),
    owners.unwrap_or(vec![ACCOUNT1, ACCOUNT2]),
    threshold.unwrap_or(2)
  )
}

fn _propose_default_change() -> DispatchResult {
  _propose_change(None, None, None, None, None, None)
}

fn _propose_change(
  origin: Option<Origin>,
  space_id: Option<SpaceId>,
  add_owners: Option<Vec<AccountId>>,
  remove_owners: Option<Vec<AccountId>>,
  new_threshold: Option<Option<u16>>,
  notes: Option<Vec<u8>>
) -> DispatchResult {

  MultiOwnership::propose_change(
    origin.unwrap_or(Origin::signed(ACCOUNT1)),
    space_id.unwrap_or(1),
    add_owners.unwrap_or(vec![ACCOUNT3]),
    remove_owners.unwrap_or(vec![]),
    new_threshold.unwrap_or(Some(3)),
    notes.unwrap_or(b"Default change proposal".to_vec())
  )
}

fn _confirm_default_change() -> DispatchResult {
  _confirm_change(None, None, None)
}

fn _confirm_change(
  origin: Option<Origin>,
  space_id: Option<SpaceId>,
  tx_id: Option<TransactionId>
) -> DispatchResult {

  MultiOwnership::confirm_change(
    origin.unwrap_or(Origin::signed(ACCOUNT2)),
    space_id.unwrap_or(1),
    tx_id.unwrap_or(1),
  )
}
