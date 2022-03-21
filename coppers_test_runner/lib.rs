#![feature(custom_test_frameworks)]
#![feature(test)]
#![test_runner(crate::runner)]

// Normally, we can use `use` to import a crate. However, the `test` crate is shipped
// with Rust itself and thus it needs to be imported with `extern crate` because it is
// a so called `sysroot` crate.
// See https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html for more
// information.
extern crate test;

mod test_runner;

// Export the runner funcion so crates that depend on this crate can use it
pub use crate::test_runner::runner as runner;
