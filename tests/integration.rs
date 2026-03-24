//! Integration tests: performance benchmarks for spec parsing and command tree generation.

use std::path::PathBuf;
use std::time::Instant;

use shrug::cmd::{router, tree};
use shrug::spec::parse_spec;

/// Load a JSON fixture from tests/fixtures/.
fn load_fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

/// Load the test Jira spec fixture with real operations.
fn load_test_spec() -> shrug::spec::model::ApiSpec {
    let spec_json = load_fixture("jira_test_spec.json");
    parse_spec(&spec_json).expect("Failed to parse test Jira spec")
}

// --- Performance benchmarks ---

#[test]
fn bench_spec_parsing() {
    let spec_json = load_fixture("jira_test_spec.json");
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = parse_spec(&spec_json).expect("Spec parse should not fail");
    }
    let elapsed = start.elapsed();

    let avg_us = elapsed.as_micros() / iterations as u128;
    eprintln!(
        "bench_spec_parsing: {} iterations in {:?} (avg {avg_us}µs/parse)",
        iterations, elapsed
    );

    assert!(
        elapsed.as_millis() < 500,
        "Spec parsing took too long: {:?} (limit 500ms for {} iterations)",
        elapsed,
        iterations
    );
}

#[test]
fn bench_command_tree_generation() {
    let spec = load_test_spec();
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let tags = router::available_tags(&spec);
        for tag in &tags {
            let _ = tree::format_operations(&spec, tag);
        }
        let _ = tree::format_tag_list(&spec);
    }
    let elapsed = start.elapsed();

    let avg_us = elapsed.as_micros() / iterations as u128;
    eprintln!(
        "bench_command_tree_generation: {} iterations in {:?} (avg {avg_us}µs/iteration)",
        iterations, elapsed
    );

    assert!(
        elapsed.as_millis() < 1000,
        "Command tree generation took too long: {:?} (limit 1000ms for {} iterations)",
        elapsed,
        iterations
    );
}
