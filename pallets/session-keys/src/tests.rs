use crate::{Error, mock::*};

use frame_support::{assert_ok, assert_noop, assert_err};
use pallet_balances::Error as BalancesError;

#[test]
fn add_key_should_work() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(_add_default_key());

        let keys = SessionKeys::key_details(ACCOUNT2).unwrap();

        assert_eq!(keys.created.account, ACCOUNT1);
        assert_eq!(keys.expires_at, BLOCK_NUMBER + 1);
        assert_eq!(keys.limit, Some(BALANCE1));
    });
}

#[test]
fn add_key_should_fail_with_zero_time_to_live() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_noop!(
            _add_key(
                None,
                None,
                Some(0),
                None
            ), Error::<Test>::ZeroTimeToLive
        );
    });
}

#[test]
fn add_key_should_fail_with_zero_limit() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_noop!(
            _add_key(
                None,
                None,
                None,
                Some(Some(0))
            ), Error::<Test>::ZeroLimit
        );
    });
}

#[test]
fn add_key_should_fail_with_session_key_already_added() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(_add_default_key());
        assert_noop!(_add_default_key() ,Error::<Test>::SessionKeyAlreadyAdded);
    });
}

#[test]
fn add_key_should_fail_with_to_many_session_keys() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(_add_default_key());
        assert_ok!(
            _add_key(
                None,
                Some(ACCOUNT3),
                None,
                None
            )
        );
        assert_noop!(
            _add_key(
                None,
                Some(ACCOUNT4),
                None,
                None
            ), Error::<Test>::TooManySessionKeys
        );
    });
}

#[test]
fn add_key_should_fail_with() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_add_default_key(), BalancesError::<Test, _>::InsufficientBalance);
    });
}

//------------------------------------------------------------------------------------------

#[test]
fn remove_key() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(_add_default_key());
        assert_ok!(_remove_default_key());

        let key = SessionKeys::keys_by_owner(ACCOUNT1);
        let details = SessionKeys::key_details(ACCOUNT2);

        assert_eq!(key.len(), 0);
        assert!(details.is_none());
    });
}

#[test]
fn remove_key_should_fail_with_session_key_not_found() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_noop!(_remove_default_key(), Error::<Test>::SessionKeyNotFound);
    });
}

//--------------------------------------------------------------------------------------------

#[test]
fn remove_keys_should_work() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(_add_default_key());
        assert_ok!(_remove_default_keys());

        let key = SessionKeys::keys_by_owner(ACCOUNT1);
        let details = SessionKeys::key_details(ACCOUNT2);

        assert_eq!(key.len(), 0);
        assert!(details.is_none());
    });
}

//---------------------------------------------------------------------------------------------

#[test]
fn proxy_should_work() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(_add_default_key());
        assert_ok!(_default_proxy());
    });
}

#[test]
fn proxy_should_fail_with_session_key_not_found() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_noop!(_default_proxy(), Error::<Test>::SessionKeyNotFound);
    });
}

#[test]
fn proxy_should_fail_with_session_key_expired() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(
            _add_key(
                None,
                None,
                Some(2),
                None
            )
        );
        System::set_block_number(3);
        assert_err!(_default_proxy(), Error::<Test>::SessionKeyExpired);
    });
}

// #[test]
// fn proxy_should_fail_with_limit_reached() {
//     ExtBuilder::build_with_balance().execute_with(|| {
//         assert_ok!(_add_default_key());
//         assert_noop!(_default_proxy(), Error::<Test>::SessionKeyExpired);
//     });
// }

