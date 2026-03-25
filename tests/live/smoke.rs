//! Smoke tests to validate the E2E harness works against live Atlassian Cloud.
//!
//! These tests use READ operations only — no resources are created or modified.

use crate::harness::{self, ShrugRunner};

#[test]
fn test_version_output() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    let result = runner.run(&["--version"]);
    result.assert_success();
    result.assert_stdout_contains("shrug");
}

#[test]
fn test_help_output() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    let result = runner.run(&["--help"]);
    result.assert_success();
    result.assert_stdout_contains("Atlassian");
}

#[test]
fn test_jira_help_loads_commands() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    let result = runner.run(&["jira", "--help"]);
    result.assert_success();
    // Static CLI should show entity subcommands
    let stdout = &result.stdout;
    assert!(
        stdout.contains("issue") || stdout.contains("project"),
        "Expected jira --help to list entity subcommands, got:\n{}",
        stdout
    );
}

#[test]
fn test_live_api_connection() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    // Create a temporary profile for the test
    let create_result = runner.run(&[
        "profile", "create", "e2e-temp",
        "--site", runner.config().site.as_str(),
        "--email", runner.config().email.as_str(),
    ]);
    assert!(
        create_result.exit_code == 0 || create_result.stderr.contains("already exists"),
        "Failed to create temp profile: {}",
        create_result.stderr
    );

    // Use project list as a basic API connectivity check
    let result = runner.run_json(&["jira", "project", "list"]);
    result.assert_success();
    assert!(
        result.json.is_some(),
        "Expected valid JSON response from live API.\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr
    );

    // Clean up temp profile
    let _ = runner.run(&["profile", "delete", "e2e-temp"]);

    harness::rate_limit_delay(runner.config());
}
