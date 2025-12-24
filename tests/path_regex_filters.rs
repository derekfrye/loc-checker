use std::process::Command;

use serde_json::Value;

#[test]
fn include_path_regex_filters_files() {
    let output = Command::new(env!("CARGO_BIN_EXE_loc-checker"))
        .args([
            "--path",
            "tests/test_proj",
            "--output-format",
            "json",
            "--include-path",
            r"src/lib\.rs$",
        ])
        .output()
        .expect("failed to run loc-checker binary");

    assert!(output.status.success(), "binary exited with failure");

    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    let json: Value = serde_json::from_str(&stdout).expect("invalid json output");

    let totals = json.get("totals").expect("missing totals section");
    assert_eq!(totals.get("files").and_then(Value::as_u64), Some(1));

    let files = json
        .get("files")
        .and_then(Value::as_array)
        .expect("missing files array");
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].get("path").and_then(Value::as_str),
        Some("src/lib.rs")
    );
}

#[test]
fn exclude_path_regex_filters_files() {
    let output = Command::new(env!("CARGO_BIN_EXE_loc-checker"))
        .args([
            "--path",
            "tests/test_proj",
            "--output-format",
            "json",
            "--exclude-path",
            r"main\.rs$",
        ])
        .output()
        .expect("failed to run loc-checker binary");

    assert!(output.status.success(), "binary exited with failure");

    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    let json: Value = serde_json::from_str(&stdout).expect("invalid json output");

    let totals = json.get("totals").expect("missing totals section");
    assert_eq!(totals.get("files").and_then(Value::as_u64), Some(1));

    let files = json
        .get("files")
        .and_then(Value::as_array)
        .expect("missing files array");
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].get("path").and_then(Value::as_str),
        Some("src/lib.rs")
    );
}
