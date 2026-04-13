use std::process::{Command, Output};

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_loc-checker");

#[test]
fn mixed_language_repo_requires_explicit_lang_when_auto_detect_is_ambiguous() {
    let output = Command::new(BIN)
        .args(["--path", "tests/mixed_proj", "--output-format", "json"])
        .output()
        .expect("failed to run loc-checker");

    assert!(!output.status.success(), "auto-detect should fail for mixed repo");

    let stderr = String::from_utf8(output.stderr).expect("stderr is not utf-8");
    assert!(
        stderr.contains("unable to auto-detect language"),
        "expected ambiguity error, got: {stderr}"
    );
}

#[test]
fn csharp_shape_fixture_reports_advanced_members() {
    let json = run_json("tests/csharp_shapes");

    assert_eq!(json.get("language").and_then(Value::as_str), Some("csharp"));

    let files = files(&json);
    assert_eq!(files.len(), 1);

    let shapes = find_file(files, "Shapes.cs");
    let summary = shapes.get("summary").expect("missing summary");

    let structs = summary
        .get("structs")
        .and_then(Value::as_array)
        .expect("missing structs");
    assert!(
        has_named_entry(structs, "Accumulator")
            && has_named_entry(structs, "PersonRecord")
            && has_named_entry(structs, "Widget")
            && has_named_entry(structs, "Notifier"),
        "expected record, struct, and class declarations"
    );

    let impl_methods = summary
        .get("impl_methods")
        .and_then(Value::as_array)
        .expect("missing impl_methods");
    assert!(
        has_method(impl_methods, "this[].get")
            && has_method(impl_methods, "operator +")
            && has_method(impl_methods, "explicit operator int")
            && has_method(impl_methods, "Description")
            && has_method(impl_methods, "Compute"),
        "expected advanced member shapes to be captured"
    );

    let trait_methods = summary
        .get("trait_methods")
        .and_then(Value::as_array)
        .expect("missing trait_methods");
    assert!(
        trait_methods.iter().any(|entry| {
            entry.get("trait_name").and_then(Value::as_str) == Some("IWorker")
                && entry.get("method_name").and_then(Value::as_str) == Some("Execute")
        }),
        "expected default interface method to be captured"
    );

    let delegates = summary
        .get("delegates")
        .and_then(Value::as_array)
        .expect("missing delegates");
    assert!(
        has_named_entry(delegates, "Processor"),
        "expected delegate declaration to be captured"
    );

    let events = summary
        .get("events")
        .and_then(Value::as_array)
        .expect("missing events");
    assert!(
        has_named_entry(events, "Changed") && has_named_entry(events, "Updated"),
        "expected event declarations to be captured"
    );
}

#[test]
fn top_level_statement_fixture_reports_aggregated_program_body() {
    let json = run_json("tests/csharp_top_level");

    let files = files(&json);
    let program = find_file(files, "Program.cs");
    let file_scope_functions = program
        .get("summary")
        .and_then(|summary| summary.get("file_scope_functions"))
        .and_then(Value::as_array)
        .expect("missing file_scope_functions");

    assert!(
        file_scope_functions.iter().any(|entry| {
            entry.get("name").and_then(Value::as_str) == Some("top-level statements")
                && entry.get("loc").and_then(Value::as_u64).is_some_and(|loc| loc >= 6)
        }),
        "expected aggregated top-level statements entry"
    );
}

#[test]
fn generated_file_matrix_excludes_common_dotnet_outputs() {
    let json = run_json("tests/csharp_generated_matrix");

    assert_eq!(json.get("language").and_then(Value::as_str), Some("csharp"));

    let files = files(&json);
    assert_eq!(files.len(), 1, "only the hand-written source should remain");
    assert_eq!(
        files[0].get("path").and_then(Value::as_str),
        Some("RealCode.cs")
    );
}

fn run_json(path: &str) -> Value {
    let output = run_command(path);
    assert!(output.status.success(), "binary exited with failure");

    let stdout = String::from_utf8(output.stdout).expect("stdout is not utf-8");
    serde_json::from_str(&stdout).expect("invalid json output")
}

fn run_command(path: &str) -> Output {
    Command::new(BIN)
        .args(["--path", path, "--output-format", "json"])
        .output()
        .expect("failed to run loc-checker")
}

fn files(json: &Value) -> &[Value] {
    json.get("files")
        .and_then(Value::as_array)
        .expect("missing files array")
}

fn find_file<'a>(files: &'a [Value], path: &str) -> &'a Value {
    files.iter()
        .find(|entry| entry.get("path").and_then(Value::as_str) == Some(path))
        .expect("missing file entry")
}

fn has_named_entry(entries: &[Value], expected_name: &str) -> bool {
    entries.iter().any(|entry| {
        entry.get("name").and_then(Value::as_str) == Some(expected_name)
    })
}

fn has_method(entries: &[Value], expected_name: &str) -> bool {
    entries.iter().any(|entry| {
        entry.get("method_name").and_then(Value::as_str) == Some(expected_name)
    })
}
