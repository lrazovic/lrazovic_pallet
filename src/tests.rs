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
fn transfer_basic_test() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::transfer(Origin::signed(1), 2, 42));
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_noop!(
            TemplateModule::transfer(Origin::signed(2), 1, 768),
            Error::<Test>::NotEnoughStakedToken
        );
        assert_ok!(TemplateModule::transfer(Origin::signed(1), 2, 256));
        assert_ok!(TemplateModule::transfer(Origin::signed(2), 1, 768));
    });
}

#[test]
fn transfer_to_self() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when you unstake more than you have.
        assert_noop!(
            TemplateModule::transfer(Origin::signed(1), 1, 1024),
            Error::<Test>::TransferToSelf
        );
    });
}

#[test]
fn transfer_too_much() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when you unstake more than you have.
        assert_noop!(
            TemplateModule::transfer(Origin::signed(1), 2, 1024),
            Error::<Test>::NotEnoughStakedToken
        );
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

#[test]
fn create_proposal_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        let hash = [0; 32];
        assert_ok!(TemplateModule::create_proposal(
            Origin::signed(1),
            hash.into(),
            42
        ));
    });
}

#[test]
fn create_proposal_not_enough_tokens() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when you unstake more than you have.
        let hash = [0; 32];
        assert_noop!(
            TemplateModule::create_proposal(Origin::signed(1), hash.into(), 1024),
            Error::<Test>::NotEnoughStakedToken
        );
    });
}
