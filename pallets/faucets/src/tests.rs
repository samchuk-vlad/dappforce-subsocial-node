use crate::{Error, mock::*, FaucetSettings, FaucetSettingsUpdate};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::DispatchError::BadOrigin;

// Add faucet
// ----------------------------------------------------------------------------

#[test]
fn add_faucet_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        let faucet_settings = Faucets::settings_by_faucet(FAUCET1).unwrap();
        assert_eq!(faucet_settings, default_faucet_settings());
    });
}

#[test]
fn add_faucet_should_fail_when_origin_is_not_root() {
    ExtBuilder::build().execute_with(|| {
        let not_root = Origin::signed(ACCOUNT1);
        assert_noop!(
            _add_faucet(Some(not_root), None),
            BadOrigin
        );
    });
}

#[test]
fn add_faucet_should_fail_when_faucet_already_added() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        assert_noop!(
            _add_default_faucet(),
            Error::<Test>::FaucetAlreadyAdded
        );
    });
}

#[test]
fn add_faucet_should_fail_when_no_free_balance_on_account() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_default_faucet());

        assert_noop!(
            _add_faucet(None, Some(FAUCET9)),
            Error::<Test>::NoFreeBalanceOnFaucet
        );
    });
}

// Update faucet
// ----------------------------------------------------------------------------

#[test]
fn update_faucet_should_work() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_ok!(_update_default_faucet());
        const SETTINGS_UPDATE: FaucetSettingsUpdate<Test> = default_faucet_settings_update();

        let faucet_settings = Faucets::settings_by_faucet(FAUCET1).unwrap();
        let updated_faucet_settings = FaucetSettings::<Test>::new(
            SETTINGS_UPDATE.period.unwrap_or(faucet_settings.period),
            SETTINGS_UPDATE.period_limit.unwrap_or(faucet_settings.period_limit),
            SETTINGS_UPDATE.drip_limit.unwrap_or(faucet_settings.drip_limit)
        );

        assert_eq!(faucet_settings.period, updated_faucet_settings.period);
        assert_eq!(faucet_settings.period_limit, updated_faucet_settings.period_limit);
        assert_eq!(faucet_settings.drip_limit, updated_faucet_settings.drip_limit);
    });
}

#[test]
fn update_faucet_should_fail_when_no_updates_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    is_active: None,
                    period: None,
                    period_limit: None,
                    drip_limit: None
                })
            ),
            Error::<Test>::NoUpdatesProvided
        );
    });
}

#[test]
fn update_faucet_should_fail_when_faucet_address_in_unknown() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _update_default_faucet(),
            Error::<Test>::FaucetNotFound
        );
    });
}

#[test]
fn update_faucet_should_fail_when_same_active_flag_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    is_active: Some(default_faucet_settings().is_active),
                    period: None,
                    period_limit: None,
                    drip_limit: None
                })
            ),
            Error::<Test>::NoUpdatesProvided
        );
    });
}

#[test]
fn update_faucet_should_fail_when_same_period_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    is_active: None,
                    period: Some(default_faucet_settings().period),
                    period_limit: None,
                    drip_limit: None
                })
            ),
            Error::<Test>::NoUpdatesProvided
        );
    });
}

#[test]
fn update_faucet_should_fail_when_same_period_limit_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    is_active: None,
                    period: None,
                    period_limit: Some(default_faucet_settings().period_limit),
                    drip_limit: None
                })
            ),
            Error::<Test>::NoUpdatesProvided
        );
    });
}

#[test]
fn update_faucet_should_fail_when_same_drip_limit_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _update_faucet(
                None,
                None,
                Some(FaucetSettingsUpdate {
                    is_active: None,
                    period: None,
                    period_limit: None,
                    drip_limit: Some(default_faucet_settings().drip_limit)
                })
            ),
            Error::<Test>::NoUpdatesProvided
        );
    });
}

// Remove faucets
// ----------------------------------------------------------------------------

#[test]
fn remove_faucets_should_work() {
    ExtBuilder::build().execute_with(|| {
        // This will add faucets with accounts ids [1; 8]
        let mut faucets = Vec::new();
        for account in FAUCET1..=FAUCET8 {
            assert_ok!(_add_faucet(None, Some(account)));
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
            assert!(Faucets::settings_by_faucet(account).is_none());
        }
        assert!(Faucets::settings_by_faucet(FAUCET8).is_some());
    });
}

#[test]
fn remove_faucets_should_handle_duplicate_addresses() {
    ExtBuilder::build().execute_with(|| {
        // This will add faucets with accounts ids [1; 8]
        let mut faucets = Vec::new();
        for account in FAUCET1..=FAUCET8 {
            assert_ok!(_add_faucet(None, Some(account)));
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
            assert!(Faucets::settings_by_faucet(account).is_none());
        }
        assert!(Faucets::settings_by_faucet(FAUCET8).is_some());
    });
}

#[test]
fn remove_faucets_should_fail_when_no_faucet_addresses_provided() {
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

// Drip
// ----------------------------------------------------------------------------

#[test]
fn drip_should_work() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_do_default_drip());

        let faucet_state = Faucets::settings_by_faucet(FAUCET1).unwrap();
        let FaucetSettings { period, drip_limit, .. } = default_faucet_settings();

        assert_eq!(faucet_state.next_period_at, INITIAL_BLOCK_NUMBER + period);
        assert_eq!(faucet_state.dripped_in_current_period, drip_limit);
        // TODO assert that ACCOUNT1 balance == drip_limit
    });
}

#[test]
fn drip_should_work_for_same_recipient_in_next_period() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        assert_ok!(_do_default_drip());

        // Move to the next period
        let FaucetSettings { period, drip_limit, .. } = default_faucet_settings();
        System::set_block_number(INITIAL_BLOCK_NUMBER + period);

        // Repeat the same drip as we did a few line above
        assert_ok!(_do_default_drip());

        let faucet_state = Faucets::settings_by_faucet(FAUCET1).unwrap();
        assert_eq!(faucet_state.next_period_at, INITIAL_BLOCK_NUMBER + period * 2);
        assert_eq!(faucet_state.dripped_in_current_period, drip_limit);
        // TODO assert that ACCOUNT1 balance == drip_limit * 2
    });
}

#[test]
fn drip_should_work_multiple_times_in_same_period() {
    ExtBuilder::build_with_one_default_drip().execute_with(|| {
        let FaucetSettings { period, drip_limit, .. } = default_faucet_settings();
        
        // Do the second drip
        assert_ok!(_drip(None, None, Some(drip_limit)));

        let faucet_state = Faucets::settings_by_faucet(FAUCET1).unwrap();
        assert_eq!(faucet_state.next_period_at, INITIAL_BLOCK_NUMBER + period);
        assert_eq!(faucet_state.dripped_in_current_period, drip_limit * 2);
        // TODO assert that ACCOUNT1 balance == drip_limit * 2
    });
}

#[test]
fn drip_should_fail_when_period_limit_reached() {
    ExtBuilder::build_with_one_default_drip().execute_with(|| {
        System::set_block_number(INITIAL_BLOCK_NUMBER);

        // Do the second drip
        assert_ok!(_do_default_drip());

        // The third drip should fail, b/c it exceeds the period limit of this faucet
        assert_noop!(
            _do_default_drip(),
            Error::<Test>::PeriodLimitReached
        );
    });
}

#[test]
fn drip_should_fail_when_zero_amount_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        assert_noop!(
            _drip(None, None, Some(0)),
            Error::<Test>::ZeroDripAmountProvided
        );
    });
}

#[test]
fn drip_should_fail_when_too_big_amount_provided() {
    ExtBuilder::build_with_faucet().execute_with(|| {
        let too_big_amount = default_faucet_settings().drip_limit + 1;
        assert_noop!(
            _drip(None, None, Some(too_big_amount)),
            Error::<Test>::DripLimitReached
        );
    });
}

// TODO test drip works when is_active == true after it was false

// TODO test drip fails when is_active == false