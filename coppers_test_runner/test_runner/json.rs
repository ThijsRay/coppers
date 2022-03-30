use git2::Repository;
use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;

use super::CompletedTest;
use super::REPEAT_TESTS_AMOUNT_OF_TIMES;

#[derive(serde::Serialize)]
struct JsonResult {
    timestamp: u64,
    head: String,
    total_us: u128,
    total_uj: u128,
    overhead_us: u128,
    overhead_uj: u128,
    number_of_repeats: usize,
    tests: Vec<CompletedTest>,
}

pub(crate) fn write_to_json(
    tests: Vec<CompletedTest>,
    total_us: u128,
    total_uj: u128,
    overhead_us: u128,
    overhead_uj: u128,
) {
    // Get git hash of last commit
    let current_directory = current_dir().unwrap();
    let current_path = current_directory.as_path().to_str().unwrap();
    let repo = Repository::open(current_path).unwrap();
    let head_hash = repo.head().unwrap().target().unwrap();
    let head_hash_bytes = head_hash.as_bytes();
    let head = hex::encode(head_hash_bytes);

    // Get the timestamp of the current time
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let output = JsonResult {
        timestamp,
        head,
        total_us,
        total_uj,
        overhead_us,
        overhead_uj,
        number_of_repeats: REPEAT_TESTS_AMOUNT_OF_TIMES,
        tests,
    };

    // Convert test results in JSON object
    let output_json = serde_json::to_string(&output).unwrap();

    let json_file_name = format!("target/coppers_results/coppers_results-{timestamp}.json");
    let mut file = File::create(json_file_name).unwrap();
    file.write_all(output_json.as_bytes()).unwrap()
}
