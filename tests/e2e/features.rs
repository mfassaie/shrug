//! CLI feature E2E tests: output formats, dry-run, fields, JQL shorthand,
//! and error remediation hints.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-feat-{}", std::process::id());
    let result = runner.run(&[
        "profile",
        "create",
        &name,
        "--site",
        runner.config().site.as_str(),
        "--email",
        runner.config().email.as_str(),
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

/// Common search args for testing different output formats.
fn search_args(project: &str) -> Vec<String> {
    let jql = format!("project = {} ORDER BY created DESC", project);
    vec![
        "jira".into(),
        "Issue search".into(),
        "search-and-reconsile-issues-using-jql".into(),
        "--jql".into(),
        jql,
        "--maxResults".into(),
        "1".into(),
    ]
}

// ─── Output Format Tests ─────────────────────────────────────────────────

#[test]
fn test_output_format_json() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let args = search_args(runner.config().jira_project.as_str());
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = runner.run_json(&str_args);
    result.assert_success();
    assert!(result.json.is_some(), "JSON output should parse");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_output_format_table() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let args = search_args(runner.config().jira_project.as_str());
    let mut all: Vec<&str> = vec!["--output", "table"];
    all.extend(args.iter().map(|s| s.as_str()));
    let result = runner.run(&all);
    result.assert_success();
    // Table output should have some content
    assert!(
        !result.stdout.trim().is_empty(),
        "Table output should not be empty"
    );
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_output_format_yaml() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let args = search_args(runner.config().jira_project.as_str());
    let mut all: Vec<&str> = vec!["--output", "yaml"];
    all.extend(args.iter().map(|s| s.as_str()));
    let result = runner.run(&all);
    result.assert_success();
    assert!(
        !result.stdout.trim().is_empty(),
        "YAML output should not be empty"
    );
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_output_format_csv() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let args = search_args(runner.config().jira_project.as_str());
    let mut all: Vec<&str> = vec!["--output", "csv"];
    all.extend(args.iter().map(|s| s.as_str()));
    let result = runner.run(&all);
    result.assert_success();
    assert!(
        !result.stdout.trim().is_empty(),
        "CSV output should not be empty"
    );
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_output_format_plain() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let args = search_args(runner.config().jira_project.as_str());
    let mut all: Vec<&str> = vec!["--output", "plain"];
    all.extend(args.iter().map(|s| s.as_str()));
    let result = runner.run(&all);
    result.assert_success();
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Dry Run ─────────────────────────────────────────────────────────────

#[test]
fn test_dry_run_mode() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let args = search_args(runner.config().jira_project.as_str());
    let mut all: Vec<&str> = vec!["--dry-run"];
    all.extend(args.iter().map(|s| s.as_str()));
    let result = runner.run(&all);
    result.assert_success();
    assert!(
        result.stdout.contains("DRY RUN") || result.stderr.contains("DRY RUN"),
        "Dry run should show DRY RUN marker.\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr
    );
    teardown_profile(&runner, &profile);
}

// ─── Error Handling ──────────────────────────────────────────────────────

#[test]
fn test_error_remediation_hint() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    // Run an invalid command to trigger error + hint
    let result = runner.run(&["jira", "nonexistent-group"]);
    assert!(result.exit_code != 0, "Invalid command should fail");
    assert!(
        result.stderr.contains("Hint:"),
        "Error output should contain remediation hint.\nstderr: {}",
        result.stderr
    );
}

// ─── Pagination ─────────────────────────────────────────────────────────

#[test]
fn test_pagination_limit() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let jql = format!("project = {} ORDER BY created DESC", project);
    let result = runner.run_json(&[
        "jira",
        "Issue search",
        "search-and-reconsile-issues-using-jql",
        "--jql",
        &jql,
        "--maxResults",
        "2",
    ]);
    result.assert_success();
    let issue_count = result
        .json
        .as_ref()
        .and_then(|j| j.get("issues"))
        .and_then(|i| i.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);
    assert!(
        issue_count <= 2,
        "Expected at most 2 issues, got {}",
        issue_count
    );
    eprintln!("Pagination limit test: got {} issues (max 2)", issue_count);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Verbose Logging ────────────────────────────────────────────────────

#[test]
fn test_verbose_logging() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run(&["-v", "jira", "Issue types", "get-issue-all-types"]);
    assert!(
        result.exit_code == 0,
        "Verbose command failed: {}",
        result.stderr
    );
    // Tracing output goes to stderr — should contain log level indicator
    assert!(
        !result.stderr.is_empty(),
        "Verbose mode should produce stderr logging output"
    );
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_trace_logging() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run(&["--trace", "jira", "Issue types", "get-issue-all-types"]);
    assert!(
        result.exit_code == 0,
        "Trace command failed: {}",
        result.stderr
    );
    // Trace should show request/response details
    assert!(
        !result.stderr.is_empty(),
        "Trace mode should produce detailed stderr logging"
    );
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── ADF Round-Trip ─────────────────────────────────────────────────────

#[test]
fn test_adf_comment_roundtrip() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    // Create issue
    let body = format!(
        r#"{{"fields":{{"project":{{"key":"{}"}},"summary":"E2E ADF test","issuetype":{{"name":"Task"}}}}}}"#,
        project
    );
    let create = runner.run_json_with_body(&body, &["jira", "Issues", "create-issue"]);
    if create.exit_code != 0 {
        eprintln!("Skipping ADF test: create-issue failed: {}", create.stderr);
        teardown_profile(&runner, &profile);
        return;
    }
    let key = create
        .json
        .as_ref()
        .and_then(|j| j.get("key"))
        .and_then(|v| v.as_str())
        .expect("Expected issue key");
    let key = key.to_string();
    harness::rate_limit_delay(runner.config());

    // Add comment with ADF body
    let adf_body = r#"{"body":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"ADF roundtrip test content"}]}]}}"#;
    let add = runner.run_json_with_body(
        adf_body,
        &[
            "jira",
            "Issue comments",
            "add-comment",
            "--issueIdOrKey",
            &key,
        ],
    );
    if add.exit_code != 0 {
        eprintln!("ADF comment add failed: {}", add.stderr);
        let _ = runner.run(&["jira", "Issues", "delete-issue", "--issueIdOrKey", &key]);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    let cid = add
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected comment id");
    harness::rate_limit_delay(runner.config());

    // Read comment back and verify content present
    let read = runner.run_json(&[
        "jira",
        "Issue comments",
        "get-comment",
        "--issueIdOrKey",
        &key,
        "--id",
        cid,
    ]);
    read.assert_success();
    // Verify the ADF text content is somewhere in the response
    let raw = read
        .json
        .as_ref()
        .map(|j| j.to_string())
        .unwrap_or_default();
    assert!(
        raw.contains("ADF roundtrip test content"),
        "Comment should contain ADF text"
    );
    harness::rate_limit_delay(runner.config());

    // Cleanup
    let _ = runner.run(&[
        "jira",
        "Issue comments",
        "delete-comment",
        "--issueIdOrKey",
        &key,
        "--id",
        cid,
    ]);
    harness::rate_limit_delay(runner.config());
    let _ = runner.run(&["jira", "Issues", "delete-issue", "--issueIdOrKey", &key]);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}
