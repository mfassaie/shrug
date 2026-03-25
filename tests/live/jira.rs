//! Comprehensive Jira live E2E tests.
//!
//! Every entity, verb, and parameter exercised against real Atlassian Cloud.
//! Create/edit commands tested with both typed params and --from-json where supported.

use std::io::Write;

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-jira-{}", std::process::id());
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

fn create_issue(runner: &ShrugRunner, project: &str, summary: &str) -> String {
    let result = runner.run_json(&[
        "jira", "issue", "create",
        "-s", summary,
        "--project", project,
        "--type", "Task",
    ]);
    result.assert_success();
    let key = result.json.as_ref()
        .and_then(|j| j.get("key"))
        .and_then(|v| v.as_str())
        .expect("Expected 'key' in create response");
    key.to_string()
}

fn delete_issue(runner: &ShrugRunner, key: &str) {
    let result = runner.run(&["jira", "issue", "delete", key, "--yes"]);
    if result.exit_code != 0 {
        eprintln!("Warning: failed to delete '{}': {}", key, result.stderr);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ISSUE — typed params
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_issue_create_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run_json(&[
        "jira", "issue", "create",
        "-s", "E2E full-param create",
        "--project", project,
        "--type", "Task",
        "--body", "Description with **markdown** formatting",
        "--assignee", "@me",
        "--priority", "Medium",
        "--label", "e2e-test",
        "--due-date", "2026-12-31",
    ]);
    result.assert_success();
    let key = result.json.as_ref()
        .and_then(|j| j.get("key"))
        .and_then(|v| v.as_str())
        .expect("Expected key");
    harness::rate_limit_delay(runner.config());

    // Verify fields were set
    let view = runner.run_json(&["jira", "issue", "view", key]);
    view.assert_success();
    let summary = view.json_field("/fields/summary").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(summary, "E2E full-param create");
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_issue_create_from_json() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    // --from-json overrides the body, but clap still requires the typed flags
    let body = format!(
        r#"{{"fields":{{"project":{{"key":"{}"}},"summary":"E2E from-json create","issuetype":{{"name":"Task"}},"priority":{{"name":"Low"}}}}}}"#,
        project
    );
    let result = runner.run_json_with_body(&body, &[
        "jira", "issue", "create",
        "-s", "ignored", "--project", project, "--type", "Task",
    ]);
    result.assert_success();
    let key = result.json.as_ref()
        .and_then(|j| j.get("key"))
        .and_then(|v| v.as_str())
        .expect("Expected key from --from-json create");
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_issue_view() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let key = create_issue(&runner, project, "E2E view test");
    harness::rate_limit_delay(runner.config());

    let view = runner.run_json(&["jira", "issue", "view", &key]);
    view.assert_success();
    assert_eq!(
        view.json_field("/fields/summary").and_then(|v| v.as_str()).unwrap_or(""),
        "E2E view test"
    );
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_issue_edit_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let key = create_issue(&runner, project, "E2E edit-params original");
    harness::rate_limit_delay(runner.config());

    let edit = runner.run(&[
        "jira", "issue", "edit", &key,
        "-s", "E2E edit-params updated",
        "--body", "Updated **description**",
        "--priority", "High",
        "--add-label", "edited",
        "--due-date", "2026-12-25",
    ]);
    assert!(edit.exit_code == 0, "edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    let verify = runner.run_json(&["jira", "issue", "view", &key]);
    verify.assert_success();
    assert_eq!(
        verify.json_field("/fields/summary").and_then(|v| v.as_str()).unwrap_or(""),
        "E2E edit-params updated"
    );
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_issue_edit_from_json() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let key = create_issue(&runner, project, "E2E json-edit original");
    harness::rate_limit_delay(runner.config());

    let body = r#"{"fields":{"summary":"E2E json-edit updated"}}"#;
    let edit = runner.run_with_body(body, &["jira", "issue", "edit", &key, "-s", "ignored"]);
    assert!(edit.exit_code == 0, "from-json edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    let verify = runner.run_json(&["jira", "issue", "view", &key]);
    verify.assert_success();
    assert_eq!(
        verify.json_field("/fields/summary").and_then(|v| v.as_str()).unwrap_or(""),
        "E2E json-edit updated"
    );
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_issue_list_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run_json(&[
        "jira", "issue", "list",
        "--project", project,
        "--type", "Task",
        "--order-by", "created DESC",
        "--fields", "key,summary,status",
    ]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from issue list");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_issue_list_with_limit() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run_json(&[
        "jira", "issue", "list",
        "--project", project,
        "--limit", "2",
    ]);
    result.assert_success();
    let count = result.json.as_ref()
        .and_then(|j| j.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert!(count <= 2, "Expected at most 2 issues, got {}", count);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_issue_delete() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let key = create_issue(&runner, project, "E2E delete target");
    harness::rate_limit_delay(runner.config());

    let del = runner.run(&["jira", "issue", "delete", &key, "--yes"]);
    del.assert_success();
    harness::rate_limit_delay(runner.config());

    // Verify gone
    let view = runner.run(&["jira", "issue", "view", &key]);
    assert!(view.exit_code != 0, "Deleted issue should not be viewable");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// COMMENT
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_comment_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();
    let key = create_issue(&runner, project, "E2E comment parent");
    harness::rate_limit_delay(runner.config());

    // CREATE with body
    let add = runner.run_json(&[
        "jira", "issue", "comment", "create", &key,
        "--body", "E2E comment with **bold** text",
    ]);
    add.assert_success();
    let cid = add.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected comment id");
    harness::rate_limit_delay(runner.config());

    // LIST
    let list = runner.run_json(&["jira", "issue", "comment", "list", &key]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira", "issue", "comment", "view", &key, cid]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // EDIT
    let edit = runner.run(&[
        "jira", "issue", "comment", "edit", &key, cid,
        "--body", "Updated comment **text**",
    ]);
    assert!(edit.exit_code == 0, "comment edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "issue", "comment", "delete", &key, cid, "--yes"]);
    assert!(del.exit_code == 0, "comment delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// WORKLOG
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_worklog_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();
    let key = create_issue(&runner, project, "E2E worklog parent");
    harness::rate_limit_delay(runner.config());

    // CREATE with all params
    let add = runner.run_json(&[
        "jira", "issue", "worklog", "create", &key,
        "--time", "1h",
        "--body", "Logged work on feature",
    ]);
    add.assert_success();
    let wid = add.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected worklog id");
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira", "issue", "worklog", "view", &key, wid]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // EDIT
    let edit = runner.run(&[
        "jira", "issue", "worklog", "edit", &key, wid,
        "--time", "2h",
        "--body", "Updated log entry",
    ]);
    assert!(edit.exit_code == 0, "worklog edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "issue", "worklog", "delete", &key, wid, "--yes"]);
    assert!(del.exit_code == 0, "worklog delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// ATTACHMENT
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_attachment_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();
    let key = create_issue(&runner, project, "E2E attachment parent");
    harness::rate_limit_delay(runner.config());

    // Create temp file
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    tmp.write_all(b"E2E test file content").expect("write");
    let path = tmp.path().to_str().unwrap().to_string();

    // CREATE (upload)
    let add = runner.run_json(&[
        "jira", "issue", "attachment", "create", &key,
        "--file", &path,
    ]);
    if add.exit_code != 0 {
        eprintln!("Attachment upload failed (may not be enabled): {}", add.stderr);
        delete_issue(&runner, &key);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    harness::rate_limit_delay(runner.config());

    // LIST
    let list = runner.run_json(&["jira", "issue", "attachment", "list", &key]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// WATCHER
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_watcher_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();
    let key = create_issue(&runner, project, "E2E watcher parent");
    harness::rate_limit_delay(runner.config());

    // CREATE (default @me)
    let add = runner.run(&["jira", "issue", "watcher", "create", &key]);
    assert!(add.exit_code == 0, "watcher create failed: {}", add.stderr);
    harness::rate_limit_delay(runner.config());

    // LIST
    let list = runner.run_json(&["jira", "issue", "watcher", "list", &key]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "issue", "watcher", "delete", &key, "--user", "@me", "--yes"]);
    assert!(del.exit_code == 0, "watcher delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// ISSUE LINK
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_link_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let key1 = create_issue(&runner, project, "E2E link source");
    harness::rate_limit_delay(runner.config());
    let key2 = create_issue(&runner, project, "E2E link target");
    harness::rate_limit_delay(runner.config());

    // CREATE with all params
    let add = runner.run(&[
        "jira", "issue", "link", "create",
        "--from", &key1,
        "--to", &key2,
        "--type", "Relates",
    ]);
    assert!(add.exit_code == 0, "link create failed: {}", add.stderr);
    harness::rate_limit_delay(runner.config());

    // LIST
    let list = runner.run_json(&["jira", "issue", "link", "list", &key1]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key2);
    harness::rate_limit_delay(runner.config());
    delete_issue(&runner, &key1);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// REMOTE LINK
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_remote_link_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();
    let key = create_issue(&runner, project, "E2E remote-link parent");
    harness::rate_limit_delay(runner.config());

    // CREATE with all params
    let add = runner.run_json(&[
        "jira", "issue", "remote-link", "create", &key,
        "--url", "https://example.com/e2e-doc",
        "--title", "E2E Design Doc",
        "--summary", "Architecture overview",
        "--relationship", "relates to",
    ]);
    add.assert_success();
    let lid = add.json.as_ref().and_then(|j| j.get("id"))
        .and_then(|v| v.as_i64())
        .expect("Expected remote link id");
    let lid_str = lid.to_string();
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira", "issue", "remote-link", "view", &key, &lid_str]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // EDIT all optional params
    let edit = runner.run(&[
        "jira", "issue", "remote-link", "edit", &key, &lid_str,
        "--title", "Updated Doc",
        "--summary", "v2 architecture",
        "--url", "https://example.com/e2e-doc-v2",
    ]);
    assert!(edit.exit_code == 0, "remote-link edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "issue", "remote-link", "delete", &key, &lid_str, "--yes"]);
    assert!(del.exit_code == 0, "remote-link delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// PROPERTY
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_property_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();
    let key = create_issue(&runner, project, "E2E property parent");
    harness::rate_limit_delay(runner.config());

    let prop_key = format!("e2e.prop.{}", std::process::id());

    // EDIT (PUT creates if not exists)
    let edit = runner.run(&[
        "jira", "issue", "property", "edit", &key, &prop_key,
        "--value", r#"{"count":42,"enabled":true}"#,
    ]);
    assert!(edit.exit_code == 0, "property edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // LIST
    let list = runner.run_json(&["jira", "issue", "property", "list", &key]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira", "issue", "property", "view", &key, &prop_key]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "issue", "property", "delete", &key, &prop_key, "--yes"]);
    assert!(del.exit_code == 0, "property delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENT
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_component_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let cname = format!("e2e-comp-{}", std::process::id());

    // CREATE with all params
    let create = runner.run_json(&[
        "jira", "project", "component", "create",
        "--name", &cname,
        "--project", project,
        "--description", "E2E test component",
        "--assignee-type", "PROJECT_DEFAULT",
    ]);
    create.assert_success();
    let cid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected component id");
    harness::rate_limit_delay(runner.config());

    // LIST
    let list = runner.run_json(&["jira", "project", "component", "list", project]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira", "project", "component", "view", cid]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // EDIT all params
    let edit = runner.run(&[
        "jira", "project", "component", "edit", cid,
        "--name", &format!("{}-upd", cname),
        "--description", "Updated component",
    ]);
    assert!(edit.exit_code == 0, "component edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "project", "component", "delete", cid, "--yes"]);
    assert!(del.exit_code == 0, "component delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// VERSION
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_version_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let vname = format!("e2e-v-{}", std::process::id());

    // CREATE with all params
    let create = runner.run_json(&[
        "jira", "project", "version", "create",
        "--name", &vname,
        "--project", project,
        "--description", "E2E release",
        "--start-date", "2026-01-01",
        "--release-date", "2026-06-30",
    ]);
    create.assert_success();
    let vid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected version id");
    harness::rate_limit_delay(runner.config());

    // LIST with params
    let list = runner.run_json(&["jira", "project", "version", "list", project]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira", "project", "version", "view", vid]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // EDIT all params
    let edit = runner.run(&[
        "jira", "project", "version", "edit", vid,
        "--name", &format!("{}-upd", vname),
        "--description", "Updated release",
        "--release-date", "2026-07-31",
    ]);
    assert!(edit.exit_code == 0, "version edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "project", "version", "delete", vid, "--yes"]);
    assert!(del.exit_code == 0, "version delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// FILTER
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_filter_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let fname = format!("e2e-filter-{}", std::process::id());
    let jql = format!("project = {}", project);

    // CREATE with all params
    let create = runner.run_json(&[
        "jira", "filter", "create",
        "--name", &fname,
        "--jql", &jql,
        "--description", "E2E test filter",
        "--favourite",
    ]);
    create.assert_success();
    let fid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected filter id");
    harness::rate_limit_delay(runner.config());

    // LIST with params
    let list = runner.run_json(&["jira", "filter", "list", "--favourites"]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira", "filter", "view", fid]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // EDIT all params
    let upd_jql = format!("project = {} AND type = Bug", project);
    let edit = runner.run(&[
        "jira", "filter", "edit", fid,
        "--name", &format!("{}-upd", fname),
        "--jql", &upd_jql,
        "--description", "Updated filter",
    ]);
    assert!(edit.exit_code == 0, "filter edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "filter", "delete", fid, "--yes"]);
    assert!(del.exit_code == 0, "filter delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// DASHBOARD
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dashboard_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let dname = format!("e2e-dash-{}", std::process::id());

    // CREATE with all params
    let create = runner.run_json(&[
        "jira", "dashboard", "create",
        "--name", &dname,
        "--description", "E2E test dashboard",
    ]);
    create.assert_success();
    let did = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str())
        .expect("Expected dashboard id");
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira", "dashboard", "view", did]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // EDIT
    let edit = runner.run(&[
        "jira", "dashboard", "edit", did,
        "--name", &format!("{}-upd", dname),
        "--description", "Updated dashboard",
    ]);
    assert!(edit.exit_code == 0, "dashboard edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE (may fail without admin)
    let del = runner.run(&["jira", "dashboard", "delete", did, "--yes"]);
    if del.exit_code != 0 {
        eprintln!("Dashboard delete failed (may need admin): {}", del.stderr);
    }
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// READ-ONLY ENTITIES
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_label_list() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["jira", "label", "list"]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_audit_list_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run(&[
        "jira", "audit", "list",
        "--from", "2026-01-01",
        "--to", "2026-12-31",
    ]);
    // Audit endpoint may fail with 403 (requires admin) or timeout
    if result.exit_code != 0 {
        eprintln!("Audit list failed (may require admin): {}", result.stderr);
    }
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_search_list_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run_json(&[
        "jira", "search", "list",
        "--project", project,
        "--type", "Task",
        "--order-by", "created DESC",
        "--fields", "key,summary,status",
    ]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from search");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_project_list_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["jira", "project", "list", "--type", "software"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from project list");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_dashboard_list_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["jira", "dashboard", "list"]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_filter_list_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run(&["jira", "filter", "list", "--favourites"]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}
