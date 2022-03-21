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

use std::any::Any;
use std::panic::catch_unwind;

pub fn runner(tests: &[&test::TestDescAndFn]) {
    println!("Running {} tests", tests.len());

    for test in tests {
        // TODO: properly handle failing tests instead of unwrapping
        run_test(test).unwrap();
    }
}

fn run_test(test: &test::TestDescAndFn) -> Result<(), ()> {
    // If a test is marked with #[ignore], it should not be executed
    if test.desc.ignore {
        Ok(())
    } else {
        print!("test {}...", test.desc.name);

        let result = match test.testfn {
            test::TestFn::StaticTestFn(f) => {
                catch_unwind(|| {
                    // TODO: Start energy measurement
                    f()
                    // TODO: Stop energy measurement
                })
            }
            _ => unimplemented!("Only StaticTestFns are supported right now"),
        };

        if test_succeeded(&test.desc, result) {
            println!("[ok]");
            Ok(())
        } else {
            println!("[err]");
            Err(())
        }
    }
}

fn test_succeeded(desc: &test::TestDesc, result: Result<(), Box<dyn Any + Send>>) -> bool {
    use test::ShouldPanic::{No, Yes, YesWithMessage};

    match desc.should_panic {
        No => result.is_ok(),
        Yes => result.is_err(),
        YesWithMessage(msg) => {
            if let Err(err) = result {
                if let Some(s) = err.downcast_ref::<&str>() {
                    return msg.eq(*s);
                }
                if let Some(s) = err.downcast_ref::<String>() {
                    return msg.eq(s);
                }
            }
            // Test failed because:
            // - It didn't panic when it should
            // - It did panic, but not with any message
            // - It did panic, but not with the correct message
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use test::TestDesc;

    fn default_test_desc() -> TestDesc {
        TestDesc {
            name: test::StaticTestName("Test"),
            ignore: false,
            ignore_message: None,
            should_panic: test::ShouldPanic::No,
            compile_fail: false,
            no_run: false,
            test_type: test::TestType::UnitTest,
        }
    }

    fn generate_panic_info(message: &'static str) -> Box<dyn Any + Send> {
        catch_unwind(|| {
            panic::panic_any(message);
        })
        .unwrap_err()
    }

    #[test]
    fn test_succeeded_succeeds_without_panic() {
        let desc = default_test_desc();
        let result = Ok(());
        assert_eq!(test_succeeded(&desc, result), true)
    }

    #[test]
    fn test_succeeded_unexpected_panic() {
        let desc = default_test_desc();
        let result = Err(generate_panic_info("Assertion failed"));
        assert_eq!(test_succeeded(&desc, result), false)
    }

    #[test]
    fn test_succeeded_expected_panic_and_did_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::Yes;
        let result = Err(generate_panic_info("Assertion failed"));
        assert_eq!(test_succeeded(&desc, result), true)
    }

    #[test]
    fn test_succeeded_expected_panic_but_did_not_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::Yes;
        let result = Ok(());
        assert_eq!(test_succeeded(&desc, result), false)
    }

    #[test]
    fn test_succeeded_expected_panic_with_str_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info("This is a message"));
        assert_eq!(test_succeeded(&desc, result), true)
    }

    #[test]
    fn test_succeeded_expected_panic_with_string_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(catch_unwind(|| {
            panic::panic_any(String::from("This is a message"));
        })
        .unwrap_err());
        assert_eq!(test_succeeded(&desc, result), true)
    }

    #[test]
    fn test_succeeded_expected_panic_with_wrong_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info("This is another message"));
        assert_eq!(test_succeeded(&desc, result), false)
    }

    #[test]
    fn test_succeeded_expected_panic_with_message_but_with_no_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info(""));
        assert_eq!(test_succeeded(&desc, result), false)
    }

    #[test]
    fn test_succeeded_expected_panic_with_message_but_did_not_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Ok(());
        assert_eq!(test_succeeded(&desc, result), false)
    }
}
