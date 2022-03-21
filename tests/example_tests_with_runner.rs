#![feature(custom_test_frameworks)]
#![test_runner(coppers::runner)]

#[test]
fn test_addition() {
    assert_eq!(1+1, 2);
}

#[test]
fn test_incorrect_addition() {
    assert_eq!(1+1, 3);
}
