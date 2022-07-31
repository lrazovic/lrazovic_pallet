use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn stake_works_for_simple_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::stake(Origin::signed(1), 42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        // assert_noop!(
        //     TemplateModule::cause_error(Origin::signed(1)),
        //     Error::<Test>::NoneValue
        // );
    });
}
