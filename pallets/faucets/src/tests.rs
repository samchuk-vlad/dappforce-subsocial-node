// Tests to be written here

use crate::{Error, mock::*, FaucetSettings, FaucetSettingsUpdate};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::DispatchError::BadOrigin;

#[test]
fn add_faucet_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        let faucet_settings = Faucet::settings_by_faucet(FAUCET1).unwrap();
        assert_eq!(faucet_settings, default_faucet_settings());
    });
}

#[test]
fn add_faucet_should_fail_bad_origin() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _add_faucet(
                Some(Origin::signed(ACCOUNT1)),
                None,
                None
            ),
            BadOrigin
        );
    });
}

#[test]
fn add_faucet_should_fail_with_faucet_already_exists() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        assert_noop!(
            _add_default_faucet(),
            Error::<Test>::FaucetAlreadyAdded
        );
    });
}

#[test]
fn add_faucet_should_fail_with_no_free_balance_on_account() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        assert_noop!(
            _add_faucet(
                None,
                Some(FAUCET9),
                None
            ),
            Error::<Test>::NoFreeBalanceOnFaucet
        );
    });
}

#[test]
fn update_faucet_should_work() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_ok!(_update_default_faucet());
        const SETTINGS_UPDATE: FaucetSettingsUpdate<BlockNumber, Balance> = default_faucet_settings_update();

        let faucet_settings = Faucet::settings_by_faucet(FAUCET1).unwrap();
        let updated_faucet_settings = FaucetSettings {
            period: SETTINGS_UPDATE.period.unwrap_or(faucet_settings.period),
            period_limit: SETTINGS_UPDATE.period_limit.unwrap_or(faucet_settings.period_limit),
            drop_limit: SETTINGS_UPDATE.drop_limit.unwrap_or(faucet_settings.drop_limit)
        };

        assert_eq!(faucet_settings.period, updated_faucet_settings.period);
        assert_eq!(faucet_settings.period_limit, updated_faucet_settings.period_limit);
        assert_eq!(faucet_settings.drop_limit, updated_faucet_settings.drop_limit);
    });
}

#[test]
fn update_faucet_should_fail_with_nothing_to_update() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    period: None,
                    period_limit: None,
                    drop_limit: None
                })
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn update_faucet_should_fail_with_faucet_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _update_default_faucet(),
            Error::<Test>::FaucetNotFound
        );
    });
}

#[test]
fn update_faucet_should_fail_with_period_not_differ() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    period: Some(default_faucet_settings().period),
                    period_limit: None,
                    drop_limit: None
                })
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn update_faucet_should_fail_with_period_limit_not_differ() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    period: None,
                    period_limit: Some(default_faucet_settings().period_limit),
                    drop_limit: None
                })
            ),
            Error::<Test>::NothingToUpdate
        );
    });
}

#[test]
fn remove_faucets_should_work() {
    ExtBuilder::build().execute_with(|| {
        // This will add faucets with accounts ids [1; 8]
        let mut faucets = Vec::new();
        for account in FAUCET1..=FAUCET8 {
            assert_ok!(_add_faucet(None, Some(account), None));
            faucets.push(account);
        }

        // This should remove only faucets from 1 to 7
        let _ = faucets.pop();
        assert_ok!(
            _remove_faucets(
                None,
                Some(faucets)
            )
        );

        for account in FAUCET1..FAUCET8 {
            assert!(Faucet::settings_by_faucet(account).is_none());
        }
        assert!(Faucet::settings_by_faucet(FAUCET8).is_some());
    });
}

#[test]
fn remove_faucets_should_work_with_duplicates() {
    ExtBuilder::build().execute_with(|| {
        // This will add faucets with accounts ids [1; 8]
        let mut faucets = Vec::new();
        for account in FAUCET1..=FAUCET8 {
            assert_ok!(_add_faucet(None, Some(account), None));
            faucets.push(account);
        }

        // This should remove only faucets from 1 to 7
        let _ = faucets.pop();
        let mut duplicates = vec![FAUCET1, FAUCET2];
        faucets.append(&mut duplicates);
        assert_ok!(
            _remove_faucets(
                None,
                Some(faucets)
            )
        );

        for account in FAUCET1..FAUCET8 {
            assert!(Faucet::settings_by_faucet(account).is_none());
        }
        assert!(Faucet::settings_by_faucet(FAUCET8).is_some());
    });
}

#[test]
fn remove_faucets_should_fail_no_faucets_specified() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _remove_faucets(
                None,
                Some(vec![])
            ),
            Error::<Test>::NoFaucetsProvided
        );
    });
}

#[test]
fn drip_should_work() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_default_drip()); // DropId 1

        let faucet_drops_info = Faucet::faucet_drops_info(FAUCET1).unwrap();
        let FaucetSettings { period, drop_limit, .. } = default_faucet_settings();

        assert_eq!(faucet_drops_info.next_period_at, INITIAL_BLOCK_NUMBER + period.unwrap());
        assert_eq!(faucet_drops_info.total_dropped, drop_limit);
        assert_eq!(Faucet::total_faucet_drops_by_account(FAUCET1, ACCOUNT1), drop_limit);
    });
}

#[test]
fn drip_should_work_twice_with_a_new_period() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_default_drip()); // DropId 1

        // Default faucet must have a limit for testing purposes
        let FaucetSettings { period, drop_limit, .. } = default_faucet_settings();
        System::set_block_number(INITIAL_BLOCK_NUMBER + period.unwrap());

        assert_ok!(_default_drip()); // Should reuse DropId 1

        let faucet_drops_info = Faucet::faucet_drops_info(FAUCET1).unwrap();
        assert_eq!(faucet_drops_info.next_period_at, INITIAL_BLOCK_NUMBER + (period.unwrap() * 2));
        assert_eq!(faucet_drops_info.total_dropped, drop_limit);
        assert_eq!(Faucet::total_faucet_drops_by_account(FAUCET1, ACCOUNT1), drop_limit * 2);
    });
}

#[test]
fn drip_should_work_with_two_drips() {
    ExtBuilder::build_with_one_default_drop().execute_with(|| {
        let FaucetSettings { period, drop_limit, .. } = default_faucet_settings();
        assert_ok!(_drip(
            None,
            Some(drop_limit),
            None
        ));


        let faucet_drops_info = Faucet::faucet_drops_info(FAUCET1).unwrap();
        assert_eq!(faucet_drops_info.next_period_at, INITIAL_BLOCK_NUMBER + period.unwrap());
        assert_eq!(faucet_drops_info.total_dropped, drop_limit * 2);
        assert_eq!(Faucet::total_faucet_drops_by_account(&FAUCET1, &ACCOUNT1), drop_limit * 2);
    });
}

#[test]
fn drip_should_fail_zero_amount() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _drip(None, Some(0), None),
            Error::<Test>::ZeroAmount
        );
    });
}

#[test]
fn drip_should_fail_faucet_limit_reached_with_same_recipient() {
    ExtBuilder::build_with_one_default_drop().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_default_drip());
        assert_noop!(
            _default_drip(),
            Error::<Test>::FaucetPeriodLimitReached
        );
    });
}

#[test]
fn drip_should_fail_with_drop_amount_limit() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _drip(None, Some(default_faucet_settings().drop_limit + 1), None),
            Error::<Test>::DropAmountLimit
        );
    });
}
