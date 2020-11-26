use crate::{Module, Trait};
use sp_core::H256;
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};
use frame_system as system;

impl_outer_origin! {
	pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
	type BaseCallFilter = ();
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
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
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
  pub const MinHandleLen: u32 = 5;
  pub const MaxHandleLen: u32 = 50;
}

impl pallet_utils::Trait for Test {
	type MinHandleLen = MinHandleLen;
	type MaxHandleLen = MaxHandleLen;
}

parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
    }

impl pallet_balances::Trait for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
}

use pallet_permissions::default_permissions::DefaultSpacePermissions;

impl pallet_permissions::Trait for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
    pub const DefaultRPCLimit: u64 = 20;
}

impl pallet_spaces::Trait for Test {
    type Event = ();
    type Roles = Roles;
    type SpaceFollows = SpaceFollows;
    type BeforeSpaceCreated = SpaceFollows;
    type AfterSpaceUpdated = ();
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
    type SpaceCreationFee = ();
    type DefaultRPCLimit = DefaultRPCLimit;
}

parameter_types! {}

impl pallet_space_follows::Trait for Test {
    type Event = ();
    type BeforeSpaceFollowed = ();
    type BeforeSpaceUnfollowed = ();
}

parameter_types! {
        pub const MaxCommentDepth: u32 = 10;
    }

impl pallet_posts::Trait for Test {
    type Event = ();
    type MaxCommentDepth = MaxCommentDepth;
    type PostScores = ();
    type AfterPostUpdated = ();
    type IsPostBlocked = Moderation;
}

parameter_types! {
        pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
    }

impl pallet_roles::Trait for Test {
    type Event = ();
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type Spaces = Spaces;
    type SpaceFollows = SpaceFollows;
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
}

parameter_types! {}

impl pallet_profiles::Trait for Test {
    type Event = ();
    type AfterProfileUpdated = ();
}

parameter_types! {
	pub const DefaultAutoblockThreshold: u16 = 20;
}

impl Trait for Test {
	type Event = ();
}

#[allow(dead_code)]
pub type Moderation = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn _new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
