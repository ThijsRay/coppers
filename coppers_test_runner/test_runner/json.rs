use git2::Repository;
use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;

use super::CompletedTest;
use super::REPEAT_TESTS_AMOUNT_OF_TIMES;

struct JsonResult {
    timestamp: u64,
    head: Vec<u8>,
    tests: Vec<CompletedTest>,
}

pub(crate) fn write_to_json(
    passed_tests: Vec<CompletedTest>,
    total_us: u128,
    total_uj: u128,
    overhead_us: u128,
    overhead_uj: u128,
) {
    // Get git hash of last commit
    // WARNING: ASSUMING REPO EXISTS!
    let current_directory = current_dir().unwrap();
    let current_path = current_directory.as_path().to_str().unwrap();
    let repo = Repository::open(current_path).unwrap();
    let head_hash = repo.head().unwrap().target().unwrap();
    let head_hash_bytes = head_hash.as_bytes();
    let head_hash_json = serde_json::to_string(&head_hash_bytes).unwrap();

    // Get the timestamp of the current time
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Convert test results in JSON object
    let json_tests = serde_json::to_string(&passed_tests).unwrap();
    let full_json = format!("{{\"timestamp\":{timestamp},\"head\":{head_hash_json},\"total_time\":{total_us},\"total_consumption\":{total_uj},\"overhead_time\":{overhead_us},\"overhead_consumption\":{overhead_uj},\"number_of_repeats\":{REPEAT_TESTS_AMOUNT_OF_TIMES},\"tests\":{json_tests}}}");

    let json_file_name = format!("target/coppers_results-{timestamp}.json");
    let mut file = File::create(json_file_name).unwrap();
    file.write_all(full_json.as_bytes())
        .expect("Writing of JSON file failed.");
}
