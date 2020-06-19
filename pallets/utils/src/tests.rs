use crate::*;
use super::{vec_remove_on, log_2};
use frame_support::{
    impl_outer_origin, parameter_types,
    weights::Weight
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_std::iter::FromIterator;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    testing::Header,
    Perbill
};

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

impl Trait for Test {
    type IpfsHashLen = IpfsHashLen;
}

type System = system::Module<Test>;
type Utils = Module<Test>;

pub type AccountId = u64;
type UsersSet = BTreeSet<User<AccountId>>;


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


const USER1: User<AccountId> = User::Account(1);
const USER2: User<AccountId> = User::Account(2);
const USER3: User<AccountId> = User::Account(3);

fn _convert_users_vec_to_btree_set(
    users_vec: Vec<User<AccountId>>
) -> Result<UsersSet, DispatchError> {
    Utils::convert_users_vec_to_btree_set(users_vec)
}


#[test]
fn log_2_should_work() {
    ExtBuilder::build().execute_with(|| {
        // None should be returned if zero(0) is provided
        assert!(log_2(0).is_none());

        // Log2 of 1 should be zero(0)
        assert_eq!(log_2(1), Some(0));

        // Log2 of 2 should be 1
        assert_eq!(log_2(2), Some(1));

        // Log2 of 128 should be 7
        assert_eq!(log_2(128), Some(7));

        // Log2 of 512 should be 9
        assert_eq!(log_2(512), Some(9));

        // Log2 of u32::MAX(4294967295) should be 31
        assert_eq!(log_2(u32::MAX), Some(31));
    });
}

#[test]
fn vec_remove_on_should_work_with_zero_elements() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![];

        vec_remove_on(vector, element);
        assert!(vector.is_empty());
    });
}

#[test]
fn vec_remove_on_should_work_with_last_element() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![6, 2];

        vector.remove(0);
        assert_eq!(vector, &mut vec![2]);

        vec_remove_on(vector, element);
        assert!(vector.is_empty());
    });
}

#[test]
fn vec_remove_on_should_work_with_two_elements() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![6, 2, 7];

        vector.remove(0);
        assert_eq!(vector, &mut vec![2, 7]);

        vec_remove_on(vector, element);
        assert_eq!(vector, &mut vec![7]);
    });
}

#[test]
fn convert_users_vec_to_btree_set_should_work() {
    ExtBuilder::build().execute_with(|| {
        // Empty vector should produce empty set
        assert_eq!(
            _convert_users_vec_to_btree_set(vec![]).ok().unwrap(),
            UsersSet::new()
        );

        assert_eq!(
            _convert_users_vec_to_btree_set(vec![USER1]).ok().unwrap(),
            UsersSet::from_iter(vec![USER1].into_iter())
        );

        // Duplicates should produce 1 unique element
        assert_eq!(
            _convert_users_vec_to_btree_set(vec![USER1, USER1, USER3]).ok().unwrap(),
            UsersSet::from_iter(vec![USER1, USER3].into_iter())
        );

        // Randomly filled vec should produce sorted set
        assert_eq!(
            _convert_users_vec_to_btree_set(vec![USER3, USER1, USER3, USER2, USER1]).ok().unwrap(),
            UsersSet::from_iter(vec![USER1, USER2, USER3].into_iter())
        );
    });
}
