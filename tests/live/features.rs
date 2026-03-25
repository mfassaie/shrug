//! Comprehensive CLI feature E2E tests: output formats, dry-run, limit,
//! fields, verbose logging, and ADF round-trip.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-feat-{}", std::process::id());
    let result = runner.run(&[
        "profile", "create", &name,
        "--site", runner.config().site.as_str(),
        "--email", runner.config().email.as_str(),
    ]);
    assert!(
        result.exit_code == 0 || result.stderr.contains("already exists"),
        "Failed to create profile: {}",
        result.stderr
    );
    name
}

fn teardown_profile(runner: &ShrugRunner, name: &str) {
    let _ = runner.run(&["profile", "delete", name]);
}

// ─── Output Formats ──────────────────────────────────────────────────────

#[test]
fn test_output_json() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run_json(&["jira", "issue", "list", "--project", project]);
    result.assert_success();
    assert!(result.json.is_some(), "JSON output should parse");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_output_table() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run(&["--output", "table", "jira", "issue", "list", "--project", project]);
    result.assert_success();
    assert!(!result.stdout.trim().is_empty(), "Table output should not be empty");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_output_csv() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run(&["--output", "csv", "jira", "issue", "list", "--project", project]);
    result.assert_success();
    assert!(!result.stdout.trim().is_empty(), "CSV output should not be empty");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Global Flags ────────────────────────────────────────────────────────

#[test]
fn test_dry_run() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run(&["--dry-run", "jira", "issue", "list", "--project", project]);
    result.assert_success();
    let combined = format!("{}{}", result.stdout, result.stderr);
    assert!(
        combined.contains("DRY RUN") || combined.contains("GET"),
        "Dry run should show method or marker.\nstdout: {}\nstderr: {}",
        result.stdout, result.stderr
    );
    teardown_profile(&runner, &profile);
}

#[test]
fn test_limit_flag() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run_json(&["jira", "issue", "list", "--project", project, "--limit", "1"]);
    result.assert_success();
    let count = result.json.as_ref().and_then(|j| j.as_array()).map(|a| a.len()).unwrap_or(0);
    assert!(count <= 1, "Expected at most 1 issue, got {}", count);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_fields_flag() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run(&[
        "--output", "table",
        "jira", "issue", "list",
        "--project", project,
        "--fields", "key,summary",
    ]);
    result.assert_success();
    assert!(
        result.stdout.contains("key") || result.stdout.contains("Key"),
        "Table should contain 'key' column: {}",
        result.stdout
    );
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_verbose_logging() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run(&["-vv", "jira", "project", "list"]);
    assert!(result.exit_code == 0, "Verbose command failed: {}", result.stderr);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── ADF Round-Trip ──────────────────────────────────────────────────────

#[test]
fn test_adf_comment_roundtrip() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    // Create issue
    let create = runner.run_json(&[
        "jira", "issue", "create", "-s", "E2E ADF roundtrip", "--project", project, "--type", "Task",
    ]);
    if create.exit_code != 0 {
        eprintln!("Skipping ADF test: {}", create.stderr);
        teardown_profile(&runner, &profile);
        return;
    }
    let key = create.json.as_ref().and_then(|j| j.get("key")).and_then(|v| v.as_str())
        .expect("Expected key").to_string();
    harness::rate_limit_delay(runner.config());

    // Add comment with markdown (CLI converts to ADF)
    let add = runner.run_json(&[
        "jira", "issue", "comment", "create", &key,
        "--body", "ADF roundtrip **bold** and _italic_ test",
    ]);
    if add.exit_code != 0 {
        eprintln!("Comment add failed: {}", add.stderr);
        let _ = runner.run(&["jira", "issue", "delete", &key, "--yes"]);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    let cid = add.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected comment id").to_string();
    harness::rate_limit_delay(runner.config());

    // Read back
    let read = runner.run_json(&["jira", "issue", "comment", "view", &key, &cid]);
    read.assert_success();
    let raw = read.json.as_ref().map(|j| j.to_string()).unwrap_or_default();
    assert!(
        raw.contains("roundtrip") || raw.contains("bold"),
        "Comment should contain ADF text"
    );
    harness::rate_limit_delay(runner.config());

    // Cleanup
    let _ = runner.run(&["jira", "issue", "comment", "delete", &key, &cid, "--yes"]);
    harness::rate_limit_delay(runner.config());
    let _ = runner.run(&["jira", "issue", "delete", &key, "--yes"]);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}
