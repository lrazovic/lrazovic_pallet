use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn stake_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::stake(Origin::signed(1), 1));
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
fn restake_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::stake(Origin::signed(1), 1));
        let block_number = System::block_number();
        System::set_block_number(block_number + 1);
        assert_ok!(TemplateModule::stake(Origin::signed(1), 1));
        let block_number = System::block_number();
        System::set_block_number(block_number + 1);
        assert_ok!(TemplateModule::unstake(Origin::signed(1), 2));
        // TODO: check that the balance is updated
    });
}

#[test]
fn unstake_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::stake(Origin::signed(1), 1));
        let block_number = System::block_number();
        System::set_block_number(block_number + 3);
        assert_ok!(TemplateModule::unstake(Origin::signed(1), 1));
    });
}

#[test]
fn transfer_basic_test() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::stake(Origin::signed(1), 42));
        assert_ok!(TemplateModule::transfer(Origin::signed(1), 2, 1));
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_noop!(
            TemplateModule::transfer(Origin::signed(2), 1, 2),
            Error::<Test>::NotEnoughStakedToken
        );
        assert_ok!(TemplateModule::stake(Origin::signed(1), 42));
        assert_ok!(TemplateModule::transfer(Origin::signed(1), 2, 3));
        assert_ok!(TemplateModule::transfer(Origin::signed(2), 1, 2));
    });
}

#[test]
fn complex_transfer_works() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_noop!(
            TemplateModule::transfer(Origin::signed(2), 1, 2),
            Error::<Test>::NotEnoughStakedToken
        );
        assert_ok!(TemplateModule::stake(Origin::signed(1), 42));
        assert_ok!(TemplateModule::transfer(Origin::signed(1), 2, 42));
        assert_ok!(TemplateModule::transfer(Origin::signed(2), 1, 2));
        let block_number = System::block_number();
        System::set_block_number(block_number + 1);
        assert_ok!(TemplateModule::unstake(Origin::signed(2), 40));
    });
}

#[test]
fn transfer_to_self() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when you unstake more than you have.
        assert_noop!(
            TemplateModule::transfer(Origin::signed(1), 1, 1),
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
fn percentage_works() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when you unstake more than you have.
        assert_ok!(TemplateModule::stake(Origin::signed(1), 100));
        assert_ok!(TemplateModule::transfer(Origin::signed(1), 2, 101));
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
        assert_ok!(TemplateModule::stake(Origin::signed(1), 42));
        assert_ok!(TemplateModule::create_proposal(
            Origin::signed(1),
            hash.into(),
            1
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

#[test]
fn change_percentage() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when you unstake more than you have.
        assert_ok!(TemplateModule::stake(Origin::signed(1), 100));
        assert_ok!(TemplateModule::transfer(Origin::signed(1), 2, 101));
        assert_noop!(
            TemplateModule::transfer(Origin::signed(1), 2, 102),
            Error::<Test>::NotEnoughStakedToken
        );
        assert_ok!(TemplateModule::change_percentage(Origin::root(), 5));
        let block_number = System::block_number();
        System::set_block_number(block_number + 1);
        assert_ok!(TemplateModule::stake(Origin::signed(1), 100));
        assert_ok!(TemplateModule::transfer(Origin::signed(1), 2, 105));
    });
}
