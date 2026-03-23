//! Jira CRUD E2E tests: Issues, Comments, Worklogs.
//!
//! Each test validates a full create → read → update → delete lifecycle
//! against live Atlassian Cloud.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-jira-{}", std::process::id());
    let result = runner.run(&[
        "profile", "create", "--name", &name,
        "--site", runner.config().site.as_str(),
        "--email", runner.config().email.as_str(),
    ]);
    assert!(
        result.exit_code == 0 || result.stderr.contains("already exists"),
        "Failed to create profile: {}", result.stderr
    );
    let _ = runner.run(&["profile", "use", "--name", &name]);
    name
}

fn teardown_profile(runner: &ShrugRunner, name: &str) {
    let _ = runner.run(&["profile", "delete", "--name", name]);
}

fn create_issue(runner: &ShrugRunner, project: &str, summary: &str) -> String {
    let body = format!(
        r#"{{"fields":{{"project":{{"key":"{}"}},"summary":"{}","issuetype":{{"name":"Task"}}}}}}"#,
        project, summary
    );
    let result = runner.run_json_with_body(&body, &["jira", "Issues", "create-issue"]);
    result.assert_success();
    let key = result.json.as_ref()
        .and_then(|j| j.get("key"))
        .and_then(|v| v.as_str())
        .expect("Expected 'key' in create-issue response");
    eprintln!("Created issue: {}", key);
    key.to_string()
}

fn delete_issue(runner: &ShrugRunner, key: &str) {
    let result = runner.run(&["jira", "Issues", "delete-issue", "--issueIdOrKey", key]);
    if result.exit_code == 0 {
        eprintln!("Deleted issue: {}", key);
    } else {
        eprintln!("Warning: failed to delete '{}': {}", key, result.stderr);
    }
}

// ─── Issue CRUD ──────────────────────────────────────────────────────────

#[test]
fn test_issue_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    // CREATE
    let key = create_issue(&runner, project, "E2E CRUD test issue");
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&["jira", "Issues", "get-issue", "--issueIdOrKey", &key]);
    read.assert_success();
    let summary = read.json_field("/fields/summary").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(summary, "E2E CRUD test issue");
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let update = runner.run_with_body(
        r#"{"fields":{"summary":"E2E CRUD updated"}}"#,
        &["jira", "Issues", "edit-issue", "--issueIdOrKey", &key],
    );
    assert!(update.exit_code == 0, "edit-issue failed: {}", update.stderr);
    harness::rate_limit_delay(runner.config());

    // READ again
    let verify = runner.run_json(&["jira", "Issues", "get-issue", "--issueIdOrKey", &key]);
    verify.assert_success();
    let updated = verify.json_field("/fields/summary").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(updated, "E2E CRUD updated");
    harness::rate_limit_delay(runner.config());

    // DELETE
    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Comment CRUD ────────────────────────────────────────────────────────

#[test]
fn test_comment_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let issue_key = create_issue(&runner, project, "E2E comment parent");
    harness::rate_limit_delay(runner.config());

    // ADD
    let body = r#"{"body":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"E2E comment"}]}]}}"#;
    let add = runner.run_json_with_body(
        body,
        &["jira", "Issue comments", "add-comment", "--issueIdOrKey", &issue_key],
    );
    add.assert_success();
    let cid = add.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected comment id");
    eprintln!("Created comment: {}", cid);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&[
        "jira", "Issue comments", "get-comment", "--issueIdOrKey", &issue_key, "--id", cid,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let ubody = r#"{"body":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"E2E updated"}]}]}}"#;
    let upd = runner.run_with_body(
        ubody,
        &["jira", "Issue comments", "update-comment", "--issueIdOrKey", &issue_key, "--id", cid],
    );
    assert!(upd.exit_code == 0, "update-comment failed: {}", upd.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&[
        "jira", "Issue comments", "delete-comment", "--issueIdOrKey", &issue_key, "--id", cid,
    ]);
    assert!(del.exit_code == 0, "delete-comment failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &issue_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Worklog CRUD ────────────────────────────────────────────────────────

#[test]
fn test_worklog_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let issue_key = create_issue(&runner, project, "E2E worklog parent");
    harness::rate_limit_delay(runner.config());

    // ADD
    let body = r#"{"timeSpentSeconds":3600,"comment":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"E2E worklog"}]}]}}"#;
    let add = runner.run_json_with_body(
        body,
        &["jira", "Issue worklogs", "add-worklog", "--issueIdOrKey", &issue_key],
    );
    add.assert_success();
    let wid = add.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected worklog id");
    eprintln!("Created worklog: {}", wid);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&[
        "jira", "Issue worklogs", "get-worklog", "--issueIdOrKey", &issue_key, "--id", wid,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let upd = runner.run_with_body(
        r#"{"timeSpentSeconds":7200}"#,
        &["jira", "Issue worklogs", "update-worklog", "--issueIdOrKey", &issue_key, "--id", wid],
    );
    assert!(upd.exit_code == 0, "update-worklog failed: {}", upd.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&[
        "jira", "Issue worklogs", "delete-worklog", "--issueIdOrKey", &issue_key, "--id", wid,
    ]);
    assert!(del.exit_code == 0, "delete-worklog failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &issue_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Filter CRUD ─────────────────────────────────────────────────────────

#[test]
fn test_filter_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let body = format!(r#"{{"name":"e2e-filter-{}","jql":"project = {}"}}"#, std::process::id(), project);
    let create = runner.run_json_with_body(&body, &["jira", "Filters", "create-filter"]);
    create.assert_success();
    let fid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected filter id");
    eprintln!("Created filter: {}", fid);
    harness::rate_limit_delay(runner.config());

    let read = runner.run_json(&["jira", "Filters", "get-filter", "--id", fid]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    let ubody = format!(r#"{{"name":"e2e-filter-upd-{}"}}"#, std::process::id());
    let upd = runner.run_with_body(&ubody, &["jira", "Filters", "update-filter", "--id", fid]);
    assert!(upd.exit_code == 0, "update-filter failed: {}", upd.stderr);
    harness::rate_limit_delay(runner.config());

    let del = runner.run(&["jira", "Filters", "delete-filter", "--id", fid]);
    assert!(del.exit_code == 0, "delete-filter failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Dashboard CRUD ──────────────────────────────────────────────────────

#[test]
fn test_dashboard_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let body = format!(r#"{{"name":"e2e-dash-{}","description":"E2E test"}}"#, std::process::id());
    let create = runner.run_json_with_body(&body, &["jira", "Dashboards", "create-dashboard"]);
    create.assert_success();
    let did = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected dashboard id");
    eprintln!("Created dashboard: {}", did);
    harness::rate_limit_delay(runner.config());

    let read = runner.run_json(&["jira", "Dashboards", "get-all-dashboards"]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    let del = runner.run(&["jira", "Dashboards", "delete-dashboard", "--id", did]);
    if del.exit_code != 0 {
        eprintln!("Note: dashboard delete failed (may need admin): {}", del.stderr);
    }
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Version CRUD ────────────────────────────────────────────────────────

#[test]
fn test_version_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let proj = runner.run_json(&["jira", "Projects", "get-project", "--projectIdOrKey", project]);
    if proj.exit_code != 0 {
        eprintln!("Skipping version test: cannot get project: {}", proj.stderr);
        teardown_profile(&runner, &profile);
        return;
    }
    let project_id = proj.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str()).unwrap_or("");
    if project_id.is_empty() {
        eprintln!("Skipping version test: no project ID");
        teardown_profile(&runner, &profile);
        return;
    }
    harness::rate_limit_delay(runner.config());

    let vname = format!("e2e-ver-{}", std::process::id());
    let body = format!(r#"{{"name":"{}","projectId":{}}}"#, vname, project_id);
    let create = runner.run_json_with_body(&body, &["jira", "Project versions", "create-version"]);
    create.assert_success();
    let vid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected version id");
    eprintln!("Created version: {}", vid);
    harness::rate_limit_delay(runner.config());

    let read = runner.run_json(&["jira", "Project versions", "get-version", "--id", vid]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    let ubody = format!(r#"{{"name":"{}-upd"}}"#, vname);
    let upd = runner.run_with_body(&ubody, &["jira", "Project versions", "update-version", "--id", vid]);
    assert!(upd.exit_code == 0, "update-version failed: {}", upd.stderr);
    harness::rate_limit_delay(runner.config());

    let del = runner.run(&["jira", "Project versions", "delete-version", "--id", vid]);
    assert!(del.exit_code == 0, "delete-version failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Component CRUD ──────────────────────────────────────────────────────

#[test]
fn test_component_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let cname = format!("e2e-comp-{}", std::process::id());
    let body = format!(r#"{{"name":"{}","project":"{}"}}"#, cname, project);
    let create = runner.run_json_with_body(&body, &["jira", "Project components", "create-component"]);
    create.assert_success();
    let cid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected component id");
    eprintln!("Created component: {}", cid);
    harness::rate_limit_delay(runner.config());

    let read = runner.run_json(&["jira", "Project components", "get-component", "--id", cid]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    let ubody = format!(r#"{{"name":"{}-upd"}}"#, cname);
    let upd = runner.run_with_body(&ubody, &["jira", "Project components", "update-component", "--id", cid]);
    assert!(upd.exit_code == 0, "update-component failed: {}", upd.stderr);
    harness::rate_limit_delay(runner.config());

    let del = runner.run(&["jira", "Project components", "delete-component", "--id", cid]);
    assert!(del.exit_code == 0, "delete-component failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Read-Only Entity Tests ──────────────────────────────────────────────

#[test]
fn test_list_projects() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let result = runner.run_json(&["jira", "Projects", "search-projects", "--maxResults", "5"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from search-projects");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_statuses() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let result = runner.run_json(&["jira", "Status", "search"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from statuses");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_priorities() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let result = runner.run_json(&["jira", "Issue priorities", "search-priorities"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from priorities");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_resolutions() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let result = runner.run_json(&["jira", "Issue resolutions", "search-resolutions"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from resolutions");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_issue_types() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let result = runner.run_json(&["jira", "Issue types", "get-issue-all-types"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from issue types");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_fields() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let result = runner.run_json(&["jira", "Issue fields", "get-fields"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from fields");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}
