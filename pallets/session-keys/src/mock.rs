use crate::{Module, Trait};

use sp_std::iter::FromIterator;
use sp_io::TestExternalities;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};

use frame_system as system;
use frame_support::{
    impl_outer_origin, impl_outer_dispatch, parameter_types, assert_ok, StorageMap,
    weights::Weight,
    dispatch::{DispatchResult},
};

use pallet_permissions::{
    SpacePermission as SP,
    SpacePermissionSet,
    SpacePermissions,
};
use pallet_utils::{Content, SpaceId};
use pallet_spaces::Call as SpacesCall;

impl_outer_origin! {
	pub enum Origin for Test {}
}

impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		frame_system::System,
		pallet_balances::Balances,
		pallet_spaces::Spaces,
	}
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
    type Call = Call;
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
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
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

parameter_types! {
  pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
  pub const IpfsCidLen: u32 = 46;
  pub const MinHandleLen: u32 = 5;
  pub const MaxHandleLen: u32 = 50;
}

impl pallet_utils::Trait for Test {
    type Event = ();
    type Currency = Balances;
    type IpfsCidLen = IpfsCidLen;
    type MinHandleLen = MinHandleLen;
    type MaxHandleLen = MaxHandleLen;
}

parameter_types! {
      pub DefaultSpacePermissions: SpacePermissions = SpacePermissions {

        // No permissions disabled by default
        none: None,

        everyone: Some(SpacePermissionSet::from_iter(vec![
            SP::UpdateOwnSubspaces,
            SP::DeleteOwnSubspaces,
            SP::HideOwnSubspaces,

            SP::UpdateOwnPosts,
            SP::DeleteOwnPosts,
            SP::HideOwnPosts,

            SP::CreateComments,
            SP::UpdateOwnComments,
            SP::DeleteOwnComments,
            SP::HideOwnComments,

            SP::Upvote,
            SP::Downvote,
            SP::Share,
        ].into_iter())),

        // Followers can do everything that everyone else can.
        follower: None,

        space_owner: Some(SpacePermissionSet::from_iter(vec![
            SP::ManageRoles,
            SP::RepresentSpaceInternally,
            SP::RepresentSpaceExternally,
            SP::OverrideSubspacePermissions,
            SP::OverridePostPermissions,

            SP::CreateSubspaces,
            SP::CreatePosts,

            SP::UpdateSpace,
            SP::UpdateAnySubspace,
            SP::UpdateAnyPost,

            SP::DeleteAnySubspace,
            SP::DeleteAnyPost,

            SP::HideAnySubspace,
            SP::HideAnyPost,
            SP::HideAnyComment,

            SP::SuggestEntityStatus,
            SP::UpdateEntityStatus,

            SP::UpdateSpaceSettings,
        ].into_iter())),
      };
    }

impl pallet_permissions::Trait for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl df_traits::moderation::IsAccountBlocked for Test {
    type AccountId = u64;

    fn is_account_blocked(account: Self::AccountId, scope: SpaceId) -> bool {
        false
    }
}

parameter_types! {}

impl pallet_spaces::Trait for Test {
    type Event = ();
    type Roles = Roles;
    type SpaceFollows = SpaceFollows;
    type BeforeSpaceCreated = SpaceFollows;
    type AfterSpaceUpdated = ();
    type IsAccountBlocked = Self;
    type IsContentBlocked = ();
    type SpaceCreationWeight = ();
}

parameter_types! {}

impl pallet_space_follows::Trait for Test {
    type Event = ();
    type BeforeSpaceFollowed = ();
    type BeforeSpaceUnfollowed = ();
}

parameter_types! {
        pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
    }

impl pallet_roles::Trait for Test {
    type Event = ();
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type Spaces = Spaces;
    type SpaceFollows = SpaceFollows;
    type IsAccountBlocked = Self;
    type IsContentBlocked = ();
}

parameter_types! {}

impl pallet_profiles::Trait for Test {
    type Event = ();
    type AfterProfileUpdated = ();
}

parameter_types! {
    pub const MaxSessionKeysPerAccount: u16 = 10;
}

impl Trait for Test {
    type Event = ();
    type Call = Call;
    type MaxSessionKeysPerAccount = MaxSessionKeysPerAccount;
    type BaseFilter = ();
}

type System = system::Module<Test>;
pub(crate) type Session_keys = Module<Test>;
type Balances = pallet_balances::Module<Test>;
type SpaceFollows = pallet_space_follows::Module<Test>;
type Spaces = pallet_spaces::Module<Test>;
type Roles = pallet_roles::Module<Test>;

pub type AccountId = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u64;

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }
}

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;

pub(crate) const BALANCE1: Balance = 2;
pub(crate) const BLOCK_NUMBER: BlockNumber = 20;

pub(crate) fn valid_content_ipfs_1() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

pub(crate) fn _add_default_key() -> DispatchResult { _add_key(None, None, None, None)}

pub(crate) fn _add_key(
    origin: Option<Origin>,
    key_account: Option<AccountId>,
    time_to_live: Option<BlockNumber>,
    limit: Option<Option<Balance>>,
) -> DispatchResult {
    Session_keys::add_key(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        key_account.unwrap_or(ACCOUNT1),
        time_to_live.unwrap_or(BLOCK_NUMBER),
        limit.unwrap_or(Some(BALANCE1)),
    )
}

pub(crate) fn _remove_default_key() -> DispatchResult { _remove_key(None, None)}

pub(crate) fn _remove_key(
    origin: Option<Origin>,
    key_account: Option<AccountId>,
) -> DispatchResult {
    Session_keys::remove_key(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        key_account.unwrap_or(ACCOUNT1),
    )
}

pub(crate) fn _remove_default_keys() -> DispatchResult { _remove_keys(None)}

pub(crate) fn _remove_keys(
    origin: Option<Origin>
) -> DispatchResult {
    Session_keys::remove_keys(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1))
    )
}

pub(crate) fn _default_proxy() -> DispatchResult { _proxy(None, None)}

pub(crate) fn _proxy(
    origin: Option<Origin>,
    call: Option<Call>,
) -> DispatchResult {
    Session_keys::proxy(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        Box::new(call.unwrap_or(Call::Spaces(SpacesCall::create_space(Some(ACCOUNT1), None, valid_content_ipfs_1())))),
    )
}
