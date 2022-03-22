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

        match test_result(&test.desc, result) {
            Ok(_) => {
                println!("[ok]");
                Ok(())
            }
            Err(None) => {
                println!("[failed]");
                Err(())
            }
            Err(Some(msg)) => {
                println!("[failed] with message: {}", msg);
                Err(())
            }
        }
    }
}

type TestResult = Result<(), Option<String>>;

fn test_result(desc: &test::TestDesc, result: Result<(), Box<dyn Any + Send>>) -> TestResult {
    use test::ShouldPanic;

    let result = match (desc.should_panic, result) {
        (ShouldPanic::No, Ok(())) | (ShouldPanic::Yes, Err(_)) => Ok(()),
        (ShouldPanic::YesWithMessage(msg), Err(ref err)) => {
            let maybe_panic_str = err
                .downcast_ref::<String>()
                .map(|e| &**e)
                .or_else(|| err.downcast_ref::<&'static str>().copied());

            if maybe_panic_str.map(|e| e.contains(msg)).unwrap_or(false) {
                Ok(())
            } else if let Some(panic_str) = maybe_panic_str {
                Err(Some(format!(
                    r#"panic did not contain expected string
      panic message: `{:?}`,
 expected substring: `{:?}`"#,
                    panic_str, msg
                )))
            } else {
                Err(Some(format!(
                    r#"expected panic with string value,
 found non-string value: `{:?}`
     expected substring: `{:?}`"#,
                    (**err).type_id(),
                    msg
                )))
            }
        }
        (ShouldPanic::Yes, Ok(())) | (ShouldPanic::YesWithMessage(_), Ok(())) => {
            Err(Some("test did not panic as expected".to_string()))
        }
        _ => Err(None)
    };

    result
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
        assert_eq!(test_result(&desc, result), Ok(()))
    }

    #[test]
    fn test_succeeded_unexpected_panic() {
        let desc = default_test_desc();
        let panic_str = "Assertion failed";
        let result = Err(generate_panic_info(panic_str));
        let test_result = test_result(&desc, result);
        if Err(None) != test_result {
            panic!("Result was {:?}", test_result)
        }
    }

    #[test]
    fn test_succeeded_expected_panic_and_did_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::Yes;
        let result = Err(generate_panic_info("Assertion failed"));
        let test_result = test_result(&desc, result);
        assert_eq!(test_result, Ok(()))
    }

    #[test]
    fn test_succeeded_expected_panic_but_did_not_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::Yes;
        let result = Ok(());
        let test_result = test_result(&desc, result);
        match test_result {
            Err(Some(msg)) => assert!(msg.contains("test did not panic")),
            _ => panic!("Result was {:?}", test_result)
        }
    }

    #[test]
    fn test_succeeded_expected_panic_with_str_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info("This is a message"));
        assert_eq!(test_result(&desc, result), Ok(()))
    }

    #[test]
    fn test_succeeded_expected_panic_with_string_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(catch_unwind(|| {
            panic::panic_any(String::from("This is a message"));
        })
        .unwrap_err());
        assert_eq!(test_result(&desc, result), Ok(()))
    }

    #[test]
    fn test_succeeded_expected_panic_with_string_message_but_got_no_string() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(catch_unwind(|| {
            panic::panic_any(123);
        })
        .unwrap_err());
        let test_result = test_result(&desc, result);
        match test_result {
            Err(Some(msg)) => assert!(msg.contains("expected panic with string value")),
            _ => panic!("Result is {:?}", test_result)
        }
    }

    #[test]
    fn test_succeeded_expected_panic_with_wrong_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info("This is another message"));
        let test_result = test_result(&desc, result);
        match test_result {
            Err(Some(msg)) => assert!(msg.contains("panic did not contain expected string")),
            _ => panic!("Result is {:?}", test_result)
        }
        
    }

    #[test]
    fn test_succeeded_expected_panic_with_message_but_with_no_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info(""));
        let test_result = test_result(&desc, result);
        match test_result {
            Err(Some(msg)) => assert!(msg.contains("panic did not contain expected string")),
            _ => panic!("Result is {:?}", test_result)
        }
    }

    #[test]
    fn test_succeeded_expected_panic_with_message_but_did_not_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Ok(());
        let test_result = test_result(&desc, result);
        match test_result {
            Err(Some(msg)) => assert!(msg.contains("test did not panic as expected")),
            _ => panic!("Result is {:?}", test_result)
        }
    }
}
