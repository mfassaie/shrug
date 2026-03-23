//! Global flag smoke tests — offline tests for --output, --color, -v, --trace, etc.
//!
//! All tests in this module run without Atlassian credentials.
//! They verify flag parsing and basic behaviour against the installed binary.

use crate::harness::SmokeRunner;

/// Generate a unique profile name with module-scoped prefix to avoid collisions.
fn unique_name(prefix: &str) -> String {
    format!("gf-{}-{}", prefix, std::process::id())
}

/// Create a test profile with dummy credentials. Caller must clean up.
fn create_test_profile(runner: &SmokeRunner, name: &str) {
    let result = runner.run(&[
        "profile",
        "create",
        "--name",
        name,
        "--site",
        "test.atlassian.net",
        "--email",
        "test@example.com",
    ]);
    assert!(
        result.exit_code == 0 || result.stderr.contains("already exists"),
        "Failed to create profile '{}': {}",
        name,
        result.stderr
    );
}

/// Delete a test profile (best-effort, no panic on failure).
fn delete_test_profile(runner: &SmokeRunner, name: &str) {
    let _ = runner.run(&["profile", "delete", "--name", name]);
}

// ─── Output Format Tests ─────────────────────────────────────────────────

#[test]
fn test_output_json() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-json");
    create_test_profile(&runner, &name);

    let result = runner.run(&["--output", "json", "profile", "list"]);
    result.assert_success();
    assert!(
        !result.stdout.trim().is_empty(),
        "JSON output should not be empty"
    );

    delete_test_profile(&runner, &name);
}

#[test]
fn test_output_table() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-table");
    create_test_profile(&runner, &name);

    let result = runner.run(&["--output", "table", "profile", "list"]);
    result.assert_success();
    assert!(
        !result.stdout.trim().is_empty(),
        "Table output should not be empty"
    );

    delete_test_profile(&runner, &name);
}

#[test]
fn test_output_csv() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-csv");
    create_test_profile(&runner, &name);

    let result = runner.run(&["--output", "csv", "profile", "list"]);
    result.assert_success();
    assert!(
        !result.stdout.trim().is_empty(),
        "CSV output should not be empty"
    );

    delete_test_profile(&runner, &name);
}

// ─── Color Flag Tests ────────────────────────────────────────────────────

#[test]
fn test_color_auto() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--color", "auto", "--help"]);
    result.assert_success();
}

#[test]
fn test_color_always() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--color", "always", "--help"]);
    result.assert_success();
}

#[test]
fn test_color_never() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--color", "never", "--help"]);
    result.assert_success();
}

// ─── Verbose and Trace Tests ─────────────────────────────────────────────

#[test]
fn test_verbose_v() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-v");
    create_test_profile(&runner, &name);

    // Single -v enables INFO level. profile list may not log at INFO,
    // so we just verify the flag is accepted (exit 0).
    let result = runner.run(&["-v", "profile", "list"]);
    result.assert_success();

    delete_test_profile(&runner, &name);
}

#[test]
fn test_verbose_vv() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-vv");
    create_test_profile(&runner, &name);

    let result = runner.run(&["-vv", "profile", "list"]);
    result.assert_success();
    assert!(
        !result.stderr.is_empty(),
        "Verbose -vv mode should produce stderr logging output"
    );

    delete_test_profile(&runner, &name);
}

#[test]
fn test_vvv_trace() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-vvv");
    create_test_profile(&runner, &name);

    let result = runner.run(&["-vvv", "profile", "list"]);
    result.assert_success();
    assert!(
        !result.stderr.is_empty(),
        "-vvv (trace) mode should produce detailed stderr logging"
    );

    delete_test_profile(&runner, &name);
}

// ─── Other Flag Tests ────────────────────────────────────────────────────

#[test]
fn test_pager_flag() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-pager");
    create_test_profile(&runner, &name);

    let result = runner.run(&["--pager", "profile", "list"]);
    result.assert_success();

    delete_test_profile(&runner, &name);
}

#[test]
fn test_dry_run_with_help() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    // --dry-run with --help is harmless and should be accepted
    let result = runner.run(&["--dry-run", "--help"]);
    result.assert_success();
}

#[test]
fn test_quiet_flag() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["-q", "--help"]);
    result.assert_success();
}
