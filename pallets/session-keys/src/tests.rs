use crate::{Error, mock::*};

use frame_support::{assert_ok, assert_noop, assert_err};
use pallet_balances::Error as BalancesError;

#[test]
fn add_key_should_work() {
    ExtBuilder::build_with_balance().execute_with(|| {
        let initial_account_balance = Balances::free_balance(ACCOUNT1);

        assert_ok!(_add_default_key());

        let keys = SessionKeys::key_details(ACCOUNT2).unwrap();
        assert_eq!(keys.created.account, ACCOUNT1);
        assert_eq!(keys.expires_at, BLOCK_NUMBER + 1);
        assert_eq!(keys.limit, Some(DEFAULT_SESSION_KEY_BALANCE));

        let account_balance_after_key_created = Balances::free_balance(ACCOUNT1);
        let session_key_balance = Balances::free_balance(ACCOUNT2);
        assert_eq!(session_key_balance, 1);
        assert_eq!(account_balance_after_key_created, initial_account_balance - session_key_balance);
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
fn add_key_should_fail_with_insufficient_balance() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_add_default_key(), BalancesError::<Test, _>::InsufficientBalance);
    });
}

//------------------------------------------------------------------------------------------

#[test]
fn remove_key_should_work() {
    ExtBuilder::build_with_balance().execute_with(|| {
        let initial_balance = Balances::free_balance(ACCOUNT1);

        assert_ok!(_add_default_key());
        assert_ok!(_remove_default_key());

        let returned_balance = Balances::free_balance(ACCOUNT1);

        assert!(SessionKeys::keys_by_owner(ACCOUNT1).is_empty());
        assert!(SessionKeys::key_details(ACCOUNT2).is_none());
        assert_eq!(initial_balance, returned_balance);
    });
}

#[test]
fn remove_key_should_fail_with_session_key_not_found() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_noop!(_remove_default_key(), Error::<Test>::SessionKeyNotFound);
    });
}

#[test]
fn remove_key_should_fail_with_not_session_key_owner() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(_add_default_key());
        assert_noop!(
            _remove_key(
                Some(Origin::signed(ACCOUNT2)),
                None
            ), Error::<Test>::NotASessionKeyOwner
        );
    });
}

//--------------------------------------------------------------------------------------------

#[test]
fn remove_keys_should_work() {
    ExtBuilder::build_with_balance().execute_with(|| {
        let initial_balance = Balances::free_balance(ACCOUNT1);

        assert_ok!(_add_default_key());
        assert_ok!(_remove_default_keys());

        let returned_balance = Balances::free_balance(ACCOUNT1);

        assert!(SessionKeys::keys_by_owner(ACCOUNT1).is_empty());
        assert!(SessionKeys::key_details(ACCOUNT2).is_none());
        assert_eq!(initial_balance, returned_balance);
    });
}

//---------------------------------------------------------------------------------------------

#[test]
fn proxy_should_work() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(_add_default_key());
        let account_balance_after_key_created = Balances::free_balance(ACCOUNT1);

        assert_ok!(_default_proxy());
        let account_balance_after_call = Balances::free_balance(ACCOUNT1);

        assert_eq!(account_balance_after_call, account_balance_after_key_created - DEFAULT_SESSION_KEY_BALANCE);

        let details = SessionKeys::key_details(ACCOUNT2).unwrap();
        assert_eq!(details.spent, DEFAULT_SESSION_KEY_BALANCE);
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

#[test]
fn proxy_should_fail_with_session_key_limit_reached() {
    ExtBuilder::build_with_balance().execute_with(|| {
        assert_ok!(
            _add_key(
                None,
                None,
                None,
                None
            )
        );
        assert_ok!(_default_proxy());
        assert_noop!(_default_proxy(), Error::<Test>::SessionKeyLimitReached);
    });
}