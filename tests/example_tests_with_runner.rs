#![feature(custom_test_frameworks)]
#![test_runner(coppers::runner)]

#[test]
fn test_succesful_test() {
    assert_eq!(1 + 1, 2);
}

#[test]
#[ignore]
fn test_ignored_test() {
    assert_eq!(5 * 10, 5)
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_should_panic_with_expected_message() {
    assert_eq!(1 + 1, 3);
}

#[test]
#[should_panic]
fn test_should_panic() {
    assert_eq!(1 + 1, 3);
}
