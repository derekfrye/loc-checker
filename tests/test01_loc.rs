use std::process::Command;

use serde_json::Value;

#[test]
fn counts_lines_for_test_project() {
    let output = Command::new(env!("CARGO_BIN_EXE_loc-checker"))
        .args(["--path", "tests/test_proj", "--output-format", "json"])
        .output()
        .expect("failed to run loc-checker binary");

    assert!(output.status.success(), "binary exited with failure");

    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    let json: Value = serde_json::from_str(&stdout).expect("invalid json output");

    let totals = json.get("totals").expect("missing totals section");
    assert_eq!(totals.get("files").and_then(Value::as_u64), Some(2));
    assert_eq!(totals.get("total_loc").and_then(Value::as_u64), Some(246));

    let files = json
        .get("files")
        .and_then(Value::as_array)
        .expect("missing files array");
    assert_eq!(files.len(), 2);

    let lib_entry = files
        .iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some("src/lib.rs"))
        .expect("missing lib.rs entry");
    let lib_summary = lib_entry.get("summary").expect("missing lib.rs summary");
    let lib_functions = lib_summary
        .get("file_scope_functions")
        .and_then(Value::as_array)
        .expect("missing lib.rs file_scope_functions");
    assert!(lib_functions.len() >= 4);
    let lib_has_expected = lib_functions.iter().any(|entry| {
        entry
            .get("name")
            .and_then(Value::as_str)
            .map(|name| name.contains("compute_series_a"))
            .unwrap_or(false)
    });
    assert!(
        lib_has_expected,
        "expected compute_series_a in lib.rs functions"
    );
    let lib_impl_methods = lib_summary
        .get("impl_methods")
        .and_then(Value::as_array)
        .expect("missing lib.rs impl_methods");
    assert!(lib_impl_methods.is_empty());

    let main_entry = files
        .iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some("src/main.rs"))
        .expect("missing main.rs entry");
    let main_summary = main_entry.get("summary").expect("missing main.rs summary");
    let main_total = main_summary
        .get("total_loc")
        .and_then(Value::as_u64)
        .expect("main.rs missing total_loc");
    assert!(main_total > 90);
    let top_functions = main_summary
        .get("top_functions")
        .and_then(Value::as_array)
        .expect("missing top_functions for main.rs");
    let has_build_full_report = top_functions.iter().any(|entry| {
        entry
            .get("name")
            .and_then(Value::as_str)
            .map(|name| name.contains("build_full_report"))
            .unwrap_or(false)
    });
    assert!(
        has_build_full_report,
        "expected build_full_report in top functions"
    );
}
