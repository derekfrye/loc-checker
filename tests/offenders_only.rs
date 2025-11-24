use std::process::Command;

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_loc-checker");
const TEST_PATH: &str = "tests/test_proj";

#[test]
fn filters_files_and_functions_based_on_thresholds() {
    let output = Command::new(BIN)
        .args([
            "--path",
            TEST_PATH,
            "--output-format",
            "json",
            "--offenders-only",
            "--offending-max-loc-per-file",
            "120",
            "--offending-max-loc-per-fn",
            "70",
        ])
        .output()
        .expect("failed to run loc-checker");

    assert!(output.status.success(), "binary exited with failure");

    let stdout = String::from_utf8(output.stdout).expect("stdout is not utf-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid json output");

    let files = json
        .get("files")
        .and_then(Value::as_array)
        .expect("missing files array");
    assert_eq!(
        files.len(),
        2,
        "expected both files to remain in offenders list"
    );

    let lib_entry = files
        .iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some("src/lib.rs"))
        .expect("missing lib.rs entry");
    let lib_functions = lib_entry
        .get("summary")
        .and_then(|summary| summary.get("file_scope_functions"))
        .and_then(Value::as_array)
        .expect("lib.rs missing file_scope_functions");
    assert!(
        lib_functions.is_empty(),
        "lib.rs should not report offending functions when only the file exceeds the limit"
    );

    let main_entry = files
        .iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some("src/main.rs"))
        .expect("missing main.rs entry");
    let main_functions = main_entry
        .get("summary")
        .and_then(|summary| summary.get("file_scope_functions"))
        .and_then(Value::as_array)
        .expect("main.rs missing file_scope_functions");
    assert_eq!(
        main_functions.len(),
        1,
        "only build_full_report should remain"
    );
    let fn_name = main_functions[0]
        .get("name")
        .and_then(Value::as_str)
        .expect("missing function name");
    assert_eq!(fn_name, "build_full_report");
}

#[test]
fn offenders_mode_can_produce_empty_results() {
    let output = Command::new(BIN)
        .args([
            "--path",
            TEST_PATH,
            "--output-format",
            "json",
            "--offenders-only",
            "--offending-max-loc-per-file",
            "1000",
            "--offending-max-loc-per-fn",
            "1000",
        ])
        .output()
        .expect("failed to run loc-checker");

    assert!(output.status.success(), "binary exited with failure");

    let stdout = String::from_utf8(output.stdout).expect("stdout is not utf-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid json output");

    let totals = json
        .get("totals")
        .expect("missing totals section")
        .as_object()
        .expect("totals is not an object");
    assert_eq!(
        totals.get("files").and_then(Value::as_u64),
        Some(0),
        "no files should exceed the generous limits"
    );
    assert_eq!(
        totals.get("total_loc").and_then(Value::as_u64),
        Some(0),
        "no LOC should be counted when no files remain"
    );

    let files = json
        .get("files")
        .and_then(Value::as_array)
        .expect("missing files array");
    assert!(
        files.is_empty(),
        "files array should be empty when no offenders are found"
    );
}
