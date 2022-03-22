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
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use test::{StaticTestFn, TestDescAndFn};

pub fn runner(tests: &[&test::TestDescAndFn]) {
    let tests: Vec<_> = tests.iter().map(make_owned_test).collect();

    println!("Running {} tests", tests.len());

    let mut ignored = 0;
    let mut filtered = 0;

    let now = Instant::now();

    let mut passed_tests = Vec::new();
    let mut failed_tests = Vec::new();

    for test in tests {
        let result = run_test(test);
        print_test_result(&result);
        match result.state {
            TestResult::Passed => passed_tests.push(result),
            TestResult::Failed(_) => failed_tests.push(result),
            TestResult::Ignored => ignored += 1,
            TestResult::Filtered => filtered += 1,
        }

        //println!("{}", result.sensor.unwrap().get_measured_uj())
    }

    let elapsed = now.elapsed();
    let seconds = elapsed.as_secs();
    let millis = elapsed.subsec_millis();

    print_failures(&failed_tests).unwrap();

    let total = passed_tests.len() + failed_tests.len();
    println!("test result: {}. {} passed; {} failed; {ignored} ignored; {filtered} filtered out; finished in {seconds}.{millis}s", passed(failed_tests.is_empty()), passed_tests.len(), failed_tests.len())
}

fn print_failures(tests: &Vec<CompletedTest>) -> std::io::Result<()> {
    if !tests.is_empty() {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        for test in tests {
            if let Some(captured) = &test.stdout {
                handle.write_fmt(format_args!("\n---- {} stdout ----\n", test.desc.name))?;
                handle.write_all(&captured)?;
                handle.write(b"\n")?;
            }
        }
        handle.write(b"\nfailures:\n")?;
        for test in tests {
            handle.write_fmt(format_args!("\t{}", test.desc.name))?;
            if let TestResult::Failed(Some(msg)) = &test.state {
                handle.write_fmt(format_args!(": {}\n", msg));
            }
        }
        handle.write(b"\n")?;
    }
    Ok(())
}

fn print_test_result(test: &CompletedTest) {
    match test.state {
        TestResult::Passed => {
            let sensor = test.sensor.as_ref().unwrap();
            let uj = (*sensor).get_measured_uj();
            let us = (*sensor).get_elapsed_time_us();
            println!("test {} ... {} - [{uj} μJ in {us} μs]", test.desc.name, passed(true))
        },
        TestResult::Failed(_) => {
            if let Some(sensor) = &test.sensor {
                let uj = (*sensor).get_measured_uj();
                let us = (*sensor).get_elapsed_time_us();
                println!("test {} ... {} - [{uj} μJ in {us} μs]", test.desc.name, passed(false))
            } else {
                println!("test {} ... {}", test.desc.name, passed(false))
            }
        },
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
    Filtered,
}

struct CompletedTest {
    desc: test::TestDesc,
    state: TestResult,
    sensor: Option<Box<dyn Sensor>>,
    stdout: Option<Vec<u8>>,
}

impl CompletedTest {
    fn empty(desc: test::TestDesc) -> Self {
        CompletedTest {
            desc,
            state: TestResult::Ignored,
            sensor: None,
            stdout: None,
        }
    }
}

fn run_test(test: test::TestDescAndFn) -> CompletedTest {
    // If a test is marked with #[ignore], it should not be executed
    if test.desc.ignore {
        CompletedTest::empty(test.desc)
    } else {
        let sensor =
            RAPLSensor::new("/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0".to_string());

        // If the sensor could not be properly created, mark the test as failed.
        if sensor.is_err() {
            return sensor
                .map_err(|err| CompletedTest {
                    state: TestResult::Failed(Some(err.to_string())),
                    ..CompletedTest::empty(test.desc)
                })
                .unwrap_err();
        }
        let mut sensor = sensor.unwrap();

        // Use internal compiler function `set_output_capture` to capture the output of the
        // tests.
        let data = Arc::new(Mutex::new(Vec::new()));
        io::set_output_capture(Some(data.clone()));

        let result = match test.testfn {
            test::TestFn::StaticTestFn(f) => catch_unwind(AssertUnwindSafe(|| {
                sensor.start_measuring();
                // Run the test function 100 times in a row
                for _ in 0..100 {
                    f();
                }
                sensor.stop_measuring();
            })),
            _ => unimplemented!("Only StaticTestFns are supported right now"),
        };

        // Reset the output capturing to the default behavior and transform the captured output
        // to a vector of bytes.
        io::set_output_capture(None);
        let stdout = Some(data.lock().unwrap_or_else(|e| e.into_inner()).to_vec());

        let state = test_state(&test.desc, result);
        CompletedTest {
            desc: test.desc,
            state,
            sensor: Some(Box::new(sensor)),
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
