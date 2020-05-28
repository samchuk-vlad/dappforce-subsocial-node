#![cfg(test)]

pub use super::*;

use sp_core::H256;
use frame_support::{impl_outer_origin, assert_ok, /*assert_noop,*/ parameter_types,
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

parameter_types! {
  pub const IpfsHashLen: u32 = 46;
}
impl pallet_utils::Trait for Test {
  type IpfsHashLen = IpfsHashLen;
}

parameter_types! {
  pub const MinHandleLen: u32 = 5;
  pub const MaxHandleLen: u32 = 50;
  pub const MinUsernameLen: u32 = 3;
  pub const MaxUsernameLen: u32 = 50;
  pub const FollowSpaceActionWeight: i16 = 7;
  pub const FollowAccountActionWeight: i16 = 3;
  pub const UpvotePostActionWeight: i16 = 5;
  pub const DownvotePostActionWeight: i16 = -3;
  pub const SharePostActionWeight: i16 = 5;
  pub const CreateCommentActionWeight: i16 = 5;
  pub const UpvoteCommentActionWeight: i16 = 4;
  pub const DownvoteCommentActionWeight: i16 = -2;
  pub const ShareCommentActionWeight: i16 = 3;
  pub const MaxCommentDepth: u32 = 10;
}
impl pallet_social::Trait for Test {
  type Event = ();
  type MinHandleLen = MinHandleLen;
  type MaxHandleLen = MaxHandleLen;
  type MinUsernameLen = MinUsernameLen;
  type MaxUsernameLen = MaxUsernameLen;
  type FollowSpaceActionWeight = FollowSpaceActionWeight;
  type FollowAccountActionWeight = FollowAccountActionWeight;
  type UpvotePostActionWeight = UpvotePostActionWeight;
  type DownvotePostActionWeight = DownvotePostActionWeight;
  type SharePostActionWeight = SharePostActionWeight;
  type CreateCommentActionWeight = CreateCommentActionWeight;
  type UpvoteCommentActionWeight = UpvoteCommentActionWeight;
  type DownvoteCommentActionWeight = DownvoteCommentActionWeight;
  type ShareCommentActionWeight = ShareCommentActionWeight;
  type MaxCommentDepth = MaxCommentDepth;
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

fn permission_manage_roles() -> /*BTreeSet<SpacePermission>*/Vec<SpacePermission> {
  /*let mut permission_set: BTreeSet<SpacePermission> = BTreeSet::new();
  permission_set.insert(SpacePermission::ManageRoles);

  permission_set*/
  vec![SpacePermission::ManageRoles]
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
                ipfs_hash: Option<Vec<u8>>, permissions: Option<Vec<SpacePermission>>) -> DispatchResult {

  Permissions::create_role(
    origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
    space_id.unwrap_or(1),
    ipfs_hash.unwrap_or_else(self::role_ipfs_hash),
    permissions.unwrap_or_else(self::permission_manage_roles)
  )
}

#[test]
fn create_role_should_work() {
  new_test_ext().execute_with(|| {
    // TODO: create a blog
    assert_ok!(_create_default_role()); // RoleId 1

    // Check whether Role is stored correctly
    assert!(Permissions::role_by_id(1).is_some());

    // Check whether data in Role structure is correct
    let role = Permissions::role_by_id(1).unwrap();
    assert_eq!(Permissions::next_role_id(), 2);

    assert!(role.updated.is_none());
    assert_eq!(role.space_id, 1);
    assert_eq!(role.disabled, false);
    assert_eq!(role.ipfs_hash, self::role_ipfs_hash());
    // assert_eq!(role.roles, self::permission_manage_roles());
  });
}
