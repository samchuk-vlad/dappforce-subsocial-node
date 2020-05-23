#![cfg(test)]

pub use super::*;

use sp_core::H256;
use frame_support::{impl_outer_origin, assert_ok, assert_noop, parameter_types,
                    weights::Weight, dispatch::DispatchResult};
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

// parameter_types! {}
impl Trait for Test {
  type Event = ();
}

type Permissions = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
fn new_test_ext() -> sp_io::TestExternalities {
  system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub type AccountId = u64;

const ACCOUNT1 : AccountId = 1;
const ACCOUNT2 : AccountId = 2;
const ACCOUNT3: AccountId = 3;

fn role_ipfs_hash() -> Vec<u8> {
  b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec()
}

fn role_update(disabled: Option<bool>, ipfs_hash: Option<Vec<u8>>, permissions: Option<BTreeSet<SpacePermission>>) -> RoleUpdate {
  RoleUpdate {
    disabled,
    ipfs_hash,
    permissions
  }
}

fn _create_default_role() -> DispatchResult {
  _create_role(None, None, None, None)
}

fn _create_role(origin: Option<Origin>, space_id: Option<SpaceId>,
                permissions: Option<BTreeSet<SpacePermission>>, ipfs_hash: Option<Vec<u8>>) -> DispatchResult {

  let mut permission_set: BTreeSet<SpacePermission> = BTreeSet::new();
  permission_set.insert(SpacePermission::ManagePosts);

  Permissions::create_role(
    origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
    space_id.unwrap_or(1),
    permissions.unwrap_or(permission_set),
    ipfs_hash.unwrap_or_else(self::role_ipfs_hash)
  )
}

#[test]
fn create_role_should_work() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_role()) // RoleId 1
  });
}
