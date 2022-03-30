// Copyright 2022 Thijs Raymakers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(test)]
#![feature(internal_output_capture)]

// Normally, we can use `use` to import a crate. However, the `test` crate is shipped
// with Rust itself and thus it needs to be imported with `extern crate` because it is
// a so called `sysroot` crate.
// See https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html for more
// information.
extern crate test;

mod test_runner;
pub(crate) mod sensors;

// Export the runner funcion so crates that depend on this crate can use it
pub use crate::test_runner::runner;
