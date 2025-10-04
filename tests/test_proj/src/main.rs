#![allow(clippy::print_stdout)]
#![allow(dead_code)]
#![allow(unused_variables)]

use test_proj::{compute_series_a, compute_series_b, generate_report};

fn main() {
    let seed = 12;
    let report = build_full_report(seed);
    for line in report {
        println!("{line}");
    }
}

/// A ton of repetition to stress our loc counting
fn build_full_report(seed: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let series_a = compute_series_a(seed);
    let series_b = compute_series_b(seed);
    let report_values = generate_report(seed);

    lines.push(format!("seed={seed}"));
    lines.push(format!("series_a={series_a}"));
    lines.push(format!("series_b={series_b}"));
    lines.push(format!("report_len={}", report_values.len()));
    lines.push(report_row("phase", "initial"));
    lines.push(report_row("stage", "collect_inputs"));
    lines.push(report_row("metric", series_a.to_string()));
    lines.push(report_row("metric", series_b.to_string()));
    lines.push(report_row("metric", report_values.iter().sum::<i32>().to_string()));
    lines.push(report_row("checkpoint", "a"));
    lines.push(report_row("checkpoint", "b"));
    lines.push(report_row("checkpoint", "c"));
    lines.push(report_row("checkpoint", "d"));
    lines.push(report_row("checkpoint", "e"));
    lines.push(report_row("checkpoint", "f"));
    lines.push(report_row("checkpoint", "g"));
    lines.push(report_row("checkpoint", "h"));
    lines.push(report_row("checkpoint", "i"));
    lines.push(report_row("checkpoint", "j"));
    lines.push(report_row("checkpoint", "k"));
    lines.push(report_row("checkpoint", "l"));
    lines.push(report_row("checkpoint", "m"));
    lines.push(report_row("checkpoint", "n"));
    lines.push(report_row("checkpoint", "o"));
    lines.push(report_row("checkpoint", "p"));
    lines.push(report_row("checkpoint", "q"));
    lines.push(report_row("checkpoint", "r"));
    lines.push(report_row("checkpoint", "s"));
    lines.push(report_row("checkpoint", "t"));
    lines.push(report_row("checkpoint", "u"));
    lines.push(report_row("checkpoint", "v"));
    lines.push(report_row("checkpoint", "w"));
    lines.push(report_row("checkpoint", "x"));
    lines.push(report_row("checkpoint", "y"));
    lines.push(report_row("checkpoint", "z"));
    lines.push(report_row("status", format_status("verifying")));
    lines.push(report_row("status", format_status("processing")));
    lines.push(report_row("status", format_status("archiving")));
    lines.push(report_row("status", format_status("complete")));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("summary", compute_section_summary(report_values.clone())));
    lines.push(report_row("closing", "ok"));
    lines.push(report_row("closing", "done"));
    lines.push(report_row("closing", "filed"));
    lines.push(report_row("closing", "archived"));
    lines.push(report_row("closing", "emailed"));
    lines.push(report_row("closing", "synced"));
    lines.push(report_row("closing", "cached"));
    lines.push(report_row("closing", "recorded"));
    lines.push(report_row("closing", "indexed"));
    lines.push(report_row("closing", "verified"));
    lines.push(report_row("closing", "acknowledged"));
    lines.push(report_row("closing", "finalized"));

    lines
}

fn report_row(key: &str, value: impl Into<String>) -> String {
    format!("{key}:{value}")
}

fn format_status(name: &str) -> String {
    format!("status::{name}")
}

fn compute_section_summary(values: Vec<i32>) -> String {
    let sum: i32 = values.iter().sum();
    let max = values.iter().max().copied().unwrap_or_default();
    let min = values.iter().min().copied().unwrap_or_default();
    format!("sum={sum},max={max},min={min}")
}
