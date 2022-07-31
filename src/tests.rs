use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn stake_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::stake(Origin::signed(1), 42));
    });
}

#[test]
fn stake_too_much_tokens() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when you unstake more than you have.
        assert_noop!(
            TemplateModule::stake(Origin::signed(1), 1024),
            Error::<Test>::NotEnoughMainToken
        );
    });
}

#[test]
fn unstake_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::unstake(Origin::signed(1), 42));
    });
}

#[test]
fn unstake_too_much_tokens() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when you unstake more than you have.
        assert_noop!(
            TemplateModule::unstake(Origin::signed(1), 1024),
            Error::<Test>::NotEnoughStakedToken
        );
    });
}
