use std::process::Command;

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_loc-checker");
const TEST_PATH: &str = "tests/csharp_proj";

#[test]
fn auto_detects_csharp_and_skips_generated_files() {
    let output = Command::new(BIN)
        .args(["--path", TEST_PATH, "--output-format", "json"])
        .output()
        .expect("failed to run loc-checker");

    assert!(output.status.success(), "binary exited with failure");

    let stdout = String::from_utf8(output.stdout).expect("stdout is not utf-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid json output");

    assert_eq!(json.get("language").and_then(Value::as_str), Some("csharp"));

    let totals = json.get("totals").expect("missing totals section");
    assert_eq!(totals.get("files").and_then(Value::as_u64), Some(2));

    let files = json
        .get("files")
        .and_then(Value::as_array)
        .expect("missing files array");
    assert_eq!(files.len(), 2);
    assert!(
        files.iter().all(|entry| {
            entry.get("path").and_then(Value::as_str).is_some_and(|path| {
                !path.ends_with(".Designer.cs") && !path.contains("/obj/")
            })
        }),
        "generated files should not appear in the report"
    );

    let program_entry = files
        .iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some("Program.cs"))
        .expect("missing Program.cs entry");
    let file_scope_functions = program_entry
        .get("summary")
        .and_then(|summary| summary.get("file_scope_functions"))
        .and_then(Value::as_array)
        .expect("Program.cs missing file_scope_functions");
    assert!(
        file_scope_functions.iter().any(|entry| {
            entry
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.contains("local LocalFormat"))
        }),
        "expected local function to be reported"
    );
}

#[test]
fn offenders_mode_reports_csharp_methods() {
    let output = Command::new(BIN)
        .args([
            "--path",
            TEST_PATH,
            "--output-format",
            "json",
            "--offenders-only",
            "--offending-max-loc-per-file",
            "10",
            "--offending-max-loc-per-fn",
            "5",
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
    assert_eq!(files.len(), 2, "expected both source files to remain");

    let helpers_entry = files
        .iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some("Helpers.cs"))
        .expect("missing Helpers.cs entry");
    let impl_methods = helpers_entry
        .get("summary")
        .and_then(|summary| summary.get("impl_methods"))
        .and_then(Value::as_array)
        .expect("Helpers.cs missing impl_methods");
    assert!(
        impl_methods.iter().any(|entry| {
            entry
                .get("method_name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == "Multiply")
        }),
        "expected Multiply to exceed the function LOC threshold"
    );
}
