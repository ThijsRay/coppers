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

// Note that this is heavily inspired by libtest that is part of the Rust language.

use coppers_sensors::{RAPLSensor, Sensor};
use std::any::Any;
use std::io::{self, Write};
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};
use test::{StaticTestFn, TestDescAndFn};

pub fn runner(tests: &[&test::TestDescAndFn]) {
    let tests: Vec<_> = tests.iter().map(make_owned_test).collect();

    println!("Running {} tests", tests.len());

    let mut ignored = 0;
    //let mut filtered = 0;

    let mut passed_tests = Vec::new();
    let mut failed_tests = Vec::new();

    let mut test_uj = 0;
    let mut test_us = 0;

    let mut sensor =
        RAPLSensor::new("/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0".to_string())
            .unwrap();
    sensor.start_measuring();

    for test in tests {
        let result = run_test(test);
        print_test_result(&result);
        match result.state {
            TestResult::Passed => {
                test_uj += result.uj.unwrap();
                test_us += result.us.unwrap();
                passed_tests.push(result);
            },
            TestResult::Failed(_) => failed_tests.push(result),
            TestResult::Ignored => ignored += 1,
            //TestResult::Filtered => filtered += 1,
        }
    }

    sensor.stop_measuring();
    let total_us = sensor.get_elapsed_time_us();
    let total_uj = sensor.get_measured_uj();

    let overhead_us = total_us - test_us;
    let overhead_uj = total_uj - test_uj;

    print_failures(&failed_tests).unwrap();

    println!("test result: {}.\n\t{} passed;\n\t{} failed;\n\t{ignored} ignored;\n\tfinished in {total_us} μs consuming {total_uj} μJ\n\tspend {test_us} μs and {test_uj} μJ on tests\n\tspend {overhead_us} μs and {overhead_uj} μJ on overhead", passed(failed_tests.is_empty()), passed_tests.len(), failed_tests.len())
}

fn print_failures(tests: &Vec<CompletedTest>) -> std::io::Result<()> {
    if !tests.is_empty() {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        for test in tests {
            if let Some(captured) = &test.stdout {
                handle.write_fmt(format_args!("\n---- {} stdout ----\n", test.desc.name))?;
                handle.write_all(captured)?;
                handle.write_all(b"\n")?;
            }
        }
        handle.write_all(b"\nfailures:\n")?;
        for test in tests {
            handle.write_fmt(format_args!("\t{}", test.desc.name))?;
            if let TestResult::Failed(Some(msg)) = &test.state {
                handle.write_fmt(format_args!(": {}\n", msg))?;
            }
        }
        handle.write_all(b"\n")?;
    }
    Ok(())
}

fn print_test_result(test: &CompletedTest) {
    match test.state {
        TestResult::Passed => {
            let uj = test.uj.unwrap();
            let us = test.us.unwrap();
            println!(
                "test {} ... {} - [{uj} μJ in {us} μs]",
                test.desc.name,
                passed(true)
            )
        }
        TestResult::Failed(_) => {
            println!("test {} ... {}", test.desc.name, passed(false))
        }
        _ => {}
    }
}

fn passed(condition: bool) -> &'static str {
    if condition {
        "ok"
    } else {
        "FAILED"
    }
}

fn make_owned_test(test: &&TestDescAndFn) -> TestDescAndFn {
    match test.testfn {
        StaticTestFn(f) => TestDescAndFn {
            testfn: StaticTestFn(f),
            desc: test.desc.clone(),
        },
        _ => panic!("non-static tests passed to test::test_main_static"),
    }
}

#[derive(Debug, PartialEq)]
enum TestResult {
    Passed,
    Failed(Option<String>),
    Ignored,
    //Filtered,
}

struct CompletedTest {
    desc: test::TestDesc,
    state: TestResult,
    uj: Option<u128>,
    us: Option<u128>,
    stdout: Option<Vec<u8>>,
}

impl CompletedTest {
    fn empty(desc: test::TestDesc) -> Self {
        CompletedTest {
            desc,
            state: TestResult::Ignored,
            uj: None,
            us: None,
            stdout: None,
        }
    }
}

fn run_test(test: test::TestDescAndFn) -> CompletedTest {
    // If a test is marked with #[ignore], it should not be executed
    if test.desc.ignore {
        CompletedTest::empty(test.desc)
    } else {
        let mut sensor =
            RAPLSensor::new("/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0".to_string())
                .unwrap();

        // Use internal compiler function `set_output_capture` to capture the output of the
        // tests.
        let data = Arc::new(Mutex::new(Vec::new()));
        io::set_output_capture(Some(data.clone()));

        let mut uj = 0;
        let mut us = 0;

        let state = match test.testfn {
            test::TestFn::StaticTestFn(f) => {
                let mut state = TestResult::Ignored;
                // Run the test function 100 times in a row
                for _ in 0..100 {
                    sensor.start_measuring();
                    let result = catch_unwind(f);
                    sensor.stop_measuring();
                    uj += sensor.get_measured_uj();
                    us += sensor.get_elapsed_time_us();

                    state = test_state(&test.desc, result);
                    if state != TestResult::Passed {
                        break;
                    }
                }
                state
            }
            _ => unimplemented!("Only StaticTestFns are supported right now"),
        };

        // Reset the output capturing to the default behavior and transform the captured output
        // to a vector of bytes.
        io::set_output_capture(None);
        let stdout = Some(data.lock().unwrap_or_else(|e| e.into_inner()).to_vec());

        CompletedTest {
            desc: test.desc,
            state,
            uj: Some(uj),
            us: Some(us),
            stdout,
        }
    }
}

fn test_state(desc: &test::TestDesc, result: Result<(), Box<dyn Any + Send>>) -> TestResult {
    use test::ShouldPanic;

    let result = match (desc.should_panic, result) {
        (ShouldPanic::No, Ok(())) | (ShouldPanic::Yes, Err(_)) => TestResult::Passed,
        (ShouldPanic::YesWithMessage(msg), Err(ref err)) => {
            let maybe_panic_str = err
                .downcast_ref::<String>()
                .map(|e| &**e)
                .or_else(|| err.downcast_ref::<&'static str>().copied());

            if maybe_panic_str.map(|e| e.contains(msg)).unwrap_or(false) {
                TestResult::Passed
            } else if let Some(panic_str) = maybe_panic_str {
                TestResult::Failed(Some(format!(
                    r#"panic did not contain expected string
      panic message: `{:?}`,
 expected substring: `{:?}`"#,
                    panic_str, msg
                )))
            } else {
                TestResult::Failed(Some(format!(
                    r#"expected panic with string value,
 found non-string value: `{:?}`
     expected substring: `{:?}`"#,
                    (**err).type_id(),
                    msg
                )))
            }
        }
        (ShouldPanic::Yes, Ok(())) | (ShouldPanic::YesWithMessage(_), Ok(())) => {
            TestResult::Failed(Some("test did not panic as expected".to_string()))
        }
        _ => TestResult::Failed(None),
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
        assert_eq!(test_state(&desc, result), TestResult::Passed)
    }

    #[test]
    fn test_succeeded_unexpected_panic() {
        let desc = default_test_desc();
        let panic_str = "Assertion failed";
        let result = Err(generate_panic_info(panic_str));
        let test_result = test_state(&desc, result);
        if TestResult::Failed(None) != test_result {
            panic!("Result was {:?}", test_result)
        }
    }

    #[test]
    fn test_succeeded_expected_panic_and_did_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::Yes;
        let result = Err(generate_panic_info("Assertion failed"));
        let test_result = test_state(&desc, result);
        assert_eq!(test_result, TestResult::Passed)
    }

    #[test]
    fn test_succeeded_expected_panic_but_did_not_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::Yes;
        let result = Ok(());
        let test_result = test_state(&desc, result);
        match test_result {
            TestResult::Failed(Some(msg)) => assert!(msg.contains("test did not panic")),
            _ => panic!("Result was {:?}", test_result),
        }
    }

    #[test]
    fn test_succeeded_expected_panic_with_str_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info("This is a message"));
        assert_eq!(test_state(&desc, result), TestResult::Passed)
    }

    #[test]
    fn test_succeeded_expected_panic_with_string_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(catch_unwind(|| {
            panic::panic_any(String::from("This is a message"));
        })
        .unwrap_err());
        assert_eq!(test_state(&desc, result), TestResult::Passed)
    }

    #[test]
    fn test_succeeded_expected_panic_with_string_message_but_got_no_string() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(catch_unwind(|| {
            panic::panic_any(123);
        })
        .unwrap_err());
        let test_result = test_state(&desc, result);
        match test_result {
            TestResult::Failed(Some(msg)) => {
                assert!(msg.contains("expected panic with string value"))
            }
            _ => panic!("Result is {:?}", test_result),
        }
    }

    #[test]
    fn test_succeeded_expected_panic_with_wrong_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info("This is another message"));
        let test_result = test_state(&desc, result);
        match test_result {
            TestResult::Failed(Some(msg)) => {
                assert!(msg.contains("panic did not contain expected string"))
            }
            _ => panic!("Result is {:?}", test_result),
        }
    }

    #[test]
    fn test_succeeded_expected_panic_with_message_but_with_no_message() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Err(generate_panic_info(""));
        let test_result = test_state(&desc, result);
        match test_result {
            TestResult::Failed(Some(msg)) => {
                assert!(msg.contains("panic did not contain expected string"))
            }
            _ => panic!("Result is {:?}", test_result),
        }
    }

    #[test]
    fn test_succeeded_expected_panic_with_message_but_did_not_panic() {
        let mut desc = default_test_desc();
        desc.should_panic = test::ShouldPanic::YesWithMessage("This is a message");
        let result = Ok(());
        let test_result = test_state(&desc, result);
        match test_result {
            TestResult::Failed(Some(msg)) => {
                assert!(msg.contains("test did not panic as expected"))
            }
            _ => panic!("Result is {:?}", test_result),
        }
    }
}
