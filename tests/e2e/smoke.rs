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
fn test_jira_help_loads_spec() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    let result = runner.run(&["jira", "--help"]);
    result.assert_success();
    // Jira spec should produce command groups from tags
    let stdout = &result.stdout;
    assert!(
        !stdout.is_empty(),
        "Expected jira --help to list command groups, got empty output"
    );
}

#[test]
fn test_live_api_connection() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    // The CLI requires a profile to exist before env var auth works.
    // Create a temporary profile, run the test, then delete it.
    let create_result = runner.run(&[
        "profile",
        "create",
        "--name",
        "e2e-temp",
        "--site",
        runner.config().site.as_str(),
        "--email",
        runner.config().email.as_str(),
    ]);
    assert!(
        create_result.exit_code == 0 || create_result.stderr.contains("already exists"),
        "Failed to create temp profile: {}",
        create_result.stderr
    );

    // Set this as the default profile
    let _ = runner.run(&["profile", "use", "--name", "e2e-temp"]);

    // Use the Jira project from E2E config to build a bounded JQL query
    let jql = format!(
        "project = {} ORDER BY created DESC",
        runner.config().jira_project
    );
    let result = runner.run_json(&[
        "jira",
        "Issue search",
        "search-and-reconsile-issues-using-jql",
        "--jql",
        &jql,
        "--maxResults",
        "1",
    ]);
    result.assert_success();
    assert!(
        result.json.is_some(),
        "Expected valid JSON response from live API.\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr
    );

    // Clean up temp profile
    let _ = runner.run(&["profile", "delete", "--name", "e2e-temp"]);

    harness::rate_limit_delay(runner.config());
}
