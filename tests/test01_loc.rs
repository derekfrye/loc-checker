use std::process::Command;

use serde_json::Value;

#[test]
fn counts_lines_for_test_project() {
    let output = Command::new(env!("CARGO_BIN_EXE_loc-checker"))
        .args([
            "--path",
            "tests/test_proj",
            "--output-format",
            "json",
        ])
        .output()
        .expect("failed to run loc-checker binary");

    assert!(output.status.success(), "binary exited with failure");

    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    let json: Value = serde_json::from_str(&stdout).expect("invalid json output");

    let totals = json
        .get("totals")
        .expect("missing totals section");
    assert_eq!(totals.get("files").and_then(Value::as_u64), Some(2));
    assert_eq!(totals.get("total_loc").and_then(Value::as_u64), Some(244));

    let files = json
        .get("files")
        .and_then(Value::as_array)
        .expect("missing files array");
    assert_eq!(files.len(), 2);

    let lib_entry = files
        .iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some("src/lib.rs"))
        .expect("missing lib.rs entry");
    let lib_functions = lib_entry
        .get("summary")
        .and_then(|summary| summary.get("function_locs"))
        .and_then(Value::as_array)
        .expect("missing lib.rs function_locs");
    assert!(lib_functions.len() >= 4);

    let main_entry = files
        .iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some("src/main.rs"))
        .expect("missing main.rs entry");
    let main_total = main_entry
        .get("summary")
        .and_then(|summary| summary.get("total_loc"))
        .and_then(Value::as_u64)
        .expect("main.rs missing total_loc");
    assert!(main_total > 90);
}
