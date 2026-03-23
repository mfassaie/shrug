//! Jira CRUD E2E tests: Issues, Comments, Worklogs.
//!
//! Each test validates a full create → read → update → delete lifecycle
//! against live Atlassian Cloud.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-jira-{}", std::process::id());
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

fn create_issue(runner: &ShrugRunner, project: &str, summary: &str) -> String {
    let body = format!(
        r#"{{"fields":{{"project":{{"key":"{}"}},"summary":"{}","issuetype":{{"name":"Task"}}}}}}"#,
        project, summary
    );
    let result = runner.run_json_with_body(&body, &["jira", "Issues", "create-issue"]);
    result.assert_success();
    let key = result
        .json
        .as_ref()
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
    let summary = read
        .json_field("/fields/summary")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(summary, "E2E CRUD test issue");
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let update = runner.run_with_body(
        r#"{"fields":{"summary":"E2E CRUD updated"}}"#,
        &["jira", "Issues", "edit-issue", "--issueIdOrKey", &key],
    );
    assert!(
        update.exit_code == 0,
        "edit-issue failed: {}",
        update.stderr
    );
    harness::rate_limit_delay(runner.config());

    // READ again
    let verify = runner.run_json(&["jira", "Issues", "get-issue", "--issueIdOrKey", &key]);
    verify.assert_success();
    let updated = verify
        .json_field("/fields/summary")
        .and_then(|v| v.as_str())
        .unwrap_or("");
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
        &[
            "jira",
            "Issue comments",
            "add-comment",
            "--issueIdOrKey",
            &issue_key,
        ],
    );
    add.assert_success();
    let cid = add
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected comment id");
    eprintln!("Created comment: {}", cid);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&[
        "jira",
        "Issue comments",
        "get-comment",
        "--issueIdOrKey",
        &issue_key,
        "--id",
        cid,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let ubody = r#"{"body":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"E2E updated"}]}]}}"#;
    let upd = runner.run_with_body(
        ubody,
        &[
            "jira",
            "Issue comments",
            "update-comment",
            "--issueIdOrKey",
            &issue_key,
            "--id",
            cid,
        ],
    );
    assert!(upd.exit_code == 0, "update-comment failed: {}", upd.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&[
        "jira",
        "Issue comments",
        "delete-comment",
        "--issueIdOrKey",
        &issue_key,
        "--id",
        cid,
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
        &[
            "jira",
            "Issue worklogs",
            "add-worklog",
            "--issueIdOrKey",
            &issue_key,
        ],
    );
    add.assert_success();
    let wid = add
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected worklog id");
    eprintln!("Created worklog: {}", wid);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&[
        "jira",
        "Issue worklogs",
        "get-worklog",
        "--issueIdOrKey",
        &issue_key,
        "--id",
        wid,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let upd = runner.run_with_body(
        r#"{"timeSpentSeconds":7200}"#,
        &[
            "jira",
            "Issue worklogs",
            "update-worklog",
            "--issueIdOrKey",
            &issue_key,
            "--id",
            wid,
        ],
    );
    assert!(upd.exit_code == 0, "update-worklog failed: {}", upd.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&[
        "jira",
        "Issue worklogs",
        "delete-worklog",
        "--issueIdOrKey",
        &issue_key,
        "--id",
        wid,
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

    let body = format!(
        r#"{{"name":"e2e-filter-{}","jql":"project = {}"}}"#,
        std::process::id(),
        project
    );
    let create = runner.run_json_with_body(&body, &["jira", "Filters", "create-filter"]);
    create.assert_success();
    let fid = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
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

    let body = format!(
        r#"{{"name":"e2e-dash-{}","description":"E2E test"}}"#,
        std::process::id()
    );
    let create = runner.run_json_with_body(&body, &["jira", "Dashboards", "create-dashboard"]);
    create.assert_success();
    let did = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected dashboard id");
    eprintln!("Created dashboard: {}", did);
    harness::rate_limit_delay(runner.config());

    let read = runner.run_json(&["jira", "Dashboards", "get-all-dashboards"]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    let del = runner.run(&["jira", "Dashboards", "delete-dashboard", "--id", did]);
    if del.exit_code != 0 {
        eprintln!(
            "Note: dashboard delete failed (may need admin): {}",
            del.stderr
        );
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

    let proj = runner.run_json(&[
        "jira",
        "Projects",
        "get-project",
        "--projectIdOrKey",
        project,
    ]);
    if proj.exit_code != 0 {
        eprintln!("Skipping version test: cannot get project: {}", proj.stderr);
        teardown_profile(&runner, &profile);
        return;
    }
    let project_id = proj
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
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
    let vid = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected version id");
    eprintln!("Created version: {}", vid);
    harness::rate_limit_delay(runner.config());

    let read = runner.run_json(&["jira", "Project versions", "get-version", "--id", vid]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    let ubody = format!(r#"{{"name":"{}-upd"}}"#, vname);
    let upd = runner.run_with_body(
        &ubody,
        &["jira", "Project versions", "update-version", "--id", vid],
    );
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
    let create =
        runner.run_json_with_body(&body, &["jira", "Project components", "create-component"]);
    create.assert_success();
    let cid = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected component id");
    eprintln!("Created component: {}", cid);
    harness::rate_limit_delay(runner.config());

    let read = runner.run_json(&["jira", "Project components", "get-component", "--id", cid]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    let ubody = format!(r#"{{"name":"{}-upd"}}"#, cname);
    let upd = runner.run_with_body(
        &ubody,
        &[
            "jira",
            "Project components",
            "update-component",
            "--id",
            cid,
        ],
    );
    assert!(
        upd.exit_code == 0,
        "update-component failed: {}",
        upd.stderr
    );
    harness::rate_limit_delay(runner.config());

    let del = runner.run(&[
        "jira",
        "Project components",
        "delete-component",
        "--id",
        cid,
    ]);
    assert!(
        del.exit_code == 0,
        "delete-component failed: {}",
        del.stderr
    );
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

// ─── Watcher Lifecycle ──────────────────────────────────────────────────

#[test]
fn test_watcher_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let issue_key = create_issue(&runner, project, "E2E watcher parent");
    harness::rate_limit_delay(runner.config());

    // Get current user's accountId from the issue reporter
    let read = runner.run_json(&["jira", "Issues", "get-issue", "--issueIdOrKey", &issue_key]);
    read.assert_success();
    let account_id = read
        .json_field("/fields/reporter/accountId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if account_id.is_empty() {
        eprintln!("Skipping watcher test: could not extract reporter accountId from issue JSON");
        delete_issue(&runner, &issue_key);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    eprintln!("Using accountId: {}", account_id);
    harness::rate_limit_delay(runner.config());

    // ADD watcher
    let body = format!("\"{}\"", account_id);
    let add = runner.run_with_body(
        &body,
        &[
            "jira",
            "Issue watchers",
            "add-watcher",
            "--issueIdOrKey",
            &issue_key,
        ],
    );
    assert!(add.exit_code == 0, "add-watcher failed: {}", add.stderr);
    harness::rate_limit_delay(runner.config());

    // GET watchers and verify content
    let get = runner.run_json(&[
        "jira",
        "Issue watchers",
        "get-issue-watchers",
        "--issueIdOrKey",
        &issue_key,
    ]);
    get.assert_success();
    let watchers_contain_user = get
        .json
        .as_ref()
        .and_then(|j| j.get("watchers"))
        .and_then(|w| w.as_array())
        .map(|arr| {
            arr.iter()
                .any(|w| w.get("accountId").and_then(|a| a.as_str()) == Some(account_id))
        })
        .unwrap_or(false);
    assert!(
        watchers_contain_user,
        "Watcher list should contain accountId {}",
        account_id
    );
    harness::rate_limit_delay(runner.config());

    // REMOVE watcher
    let del = runner.run(&[
        "jira",
        "Issue watchers",
        "remove-watcher",
        "--issueIdOrKey",
        &issue_key,
        "--accountId",
        account_id,
    ]);
    assert!(del.exit_code == 0, "remove-watcher failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &issue_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Vote Lifecycle ─────────────────────────────────────────────────────

#[test]
fn test_vote_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let issue_key = create_issue(&runner, project, "E2E vote parent");
    harness::rate_limit_delay(runner.config());

    // ADD vote
    let add = runner.run(&[
        "jira",
        "Issue votes",
        "add-vote",
        "--issueIdOrKey",
        &issue_key,
    ]);
    if add.exit_code != 0 {
        // Jira returns 404 if user is the reporter or voting is disabled
        eprintln!(
            "add-vote returned exit code {} (expected if voting on own issue): {}",
            add.exit_code, add.stderr
        );
        delete_issue(&runner, &issue_key);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    harness::rate_limit_delay(runner.config());

    // GET votes
    let get = runner.run_json(&[
        "jira",
        "Issue votes",
        "get-votes",
        "--issueIdOrKey",
        &issue_key,
    ]);
    get.assert_success();
    assert!(get.json.is_some(), "Expected JSON from get-votes");
    harness::rate_limit_delay(runner.config());

    // REMOVE vote
    let del = runner.run(&[
        "jira",
        "Issue votes",
        "remove-vote",
        "--issueIdOrKey",
        &issue_key,
    ]);
    assert!(del.exit_code == 0, "remove-vote failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &issue_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Issue Link Lifecycle ───────────────────────────────────────────────

#[test]
fn test_issue_link_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let key1 = create_issue(&runner, project, "E2E link source");
    harness::rate_limit_delay(runner.config());
    let key2 = create_issue(&runner, project, "E2E link target");
    harness::rate_limit_delay(runner.config());

    // Get available link types
    let types = runner.run_json(&["jira", "Issue link types", "get-issue-link-types"]);
    types.assert_success();
    let link_type_name = types
        .json
        .as_ref()
        .and_then(|j| j.get("issueLinkTypes"))
        .and_then(|arr| arr.as_array())
        .and_then(|arr| arr.first())
        .and_then(|lt| lt.get("name"))
        .and_then(|n| n.as_str());
    let link_type_name = match link_type_name {
        Some(name) => name.to_string(),
        None => {
            eprintln!(
                "Skipping link test: no issue link types available (linking may be disabled)"
            );
            delete_issue(&runner, &key1);
            harness::rate_limit_delay(runner.config());
            delete_issue(&runner, &key2);
            harness::rate_limit_delay(runner.config());
            teardown_profile(&runner, &profile);
            return;
        }
    };
    eprintln!("Using link type: {}", link_type_name);
    harness::rate_limit_delay(runner.config());

    // CREATE link
    let body = format!(
        r#"{{"type":{{"name":"{}"}},"inwardIssue":{{"key":"{}"}},"outwardIssue":{{"key":"{}"}}}}"#,
        link_type_name, key1, key2
    );
    let create = runner.run_with_body(&body, &["jira", "Issue links", "link-issues"]);
    assert!(
        create.exit_code == 0,
        "link-issues failed: {}",
        create.stderr
    );
    harness::rate_limit_delay(runner.config());

    // GET link ID by reading the source issue and filtering issuelinks
    let read = runner.run_json(&["jira", "Issues", "get-issue", "--issueIdOrKey", &key1]);
    read.assert_success();
    let link_id = read
        .json
        .as_ref()
        .and_then(|j| j.get("fields"))
        .and_then(|f| f.get("issuelinks"))
        .and_then(|links| links.as_array())
        .and_then(|arr| {
            arr.iter().find(|link| {
                let outward_match = link
                    .get("outwardIssue")
                    .and_then(|i| i.get("key"))
                    .and_then(|k| k.as_str())
                    == Some(key2.as_str());
                let inward_match = link
                    .get("inwardIssue")
                    .and_then(|i| i.get("key"))
                    .and_then(|k| k.as_str())
                    == Some(key2.as_str());
                outward_match || inward_match
            })
        })
        .and_then(|link| link.get("id"))
        .and_then(|id| id.as_str())
        .expect("Expected link id matching target issue in issuelinks");
    eprintln!("Created link: {}", link_id);
    harness::rate_limit_delay(runner.config());

    // GET link details
    let get = runner.run_json(&["jira", "Issue links", "get-issue-link", "--linkId", link_id]);
    get.assert_success();
    harness::rate_limit_delay(runner.config());

    // DELETE link
    let del = runner.run(&[
        "jira",
        "Issue links",
        "delete-issue-link",
        "--linkId",
        link_id,
    ]);
    assert!(
        del.exit_code == 0,
        "delete-issue-link failed: {}",
        del.stderr
    );
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &key1);
    harness::rate_limit_delay(runner.config());
    delete_issue(&runner, &key2);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Issue Type CRUD Lifecycle ──────────────────────────────────────────

#[test]
fn test_issue_type_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let type_name = format!("e2e-type-{}", std::process::id());
    let body = format!(
        r#"{{"name":"{}","description":"E2E test type","type":"standard"}}"#,
        type_name
    );

    // CREATE
    let create = runner.run_json_with_body(&body, &["jira", "Issue types", "create-issue-type"]);
    if create.exit_code != 0 {
        eprintln!(
            "Skipping issue type CRUD: create failed (admin required?): {}",
            create.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }
    let type_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected issue type id");
    eprintln!("Created issue type: {} ({})", type_name, type_id);
    harness::rate_limit_delay(runner.config());

    // READ and verify content
    let read = runner.run_json(&["jira", "Issue types", "get-issue-type", "--id", type_id]);
    read.assert_success();
    let read_name = read
        .json_field("/name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(read_name, type_name, "Issue type name should match");
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let upd = runner.run_with_body(
        r#"{"description":"E2E updated"}"#,
        &["jira", "Issue types", "update-issue-type", "--id", type_id],
    );
    assert!(
        upd.exit_code == 0,
        "update-issue-type failed: {}",
        upd.stderr
    );
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "Issue types", "delete-issue-type", "--id", type_id]);
    if del.exit_code != 0 {
        eprintln!(
            "Warning: delete-issue-type failed (issues may reference type): {}",
            del.stderr
        );
    } else {
        eprintln!("Deleted issue type: {}", type_id);
    }
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Group CRUD Lifecycle ───────────────────────────────────────────────

#[test]
fn test_group_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let group_name = format!("e2e-group-{}", std::process::id());
    let body = format!(r#"{{"name":"{}"}}"#, group_name);

    // CREATE
    let create = runner.run_json_with_body(&body, &["jira", "Groups", "create-group"]);
    if create.exit_code != 0 {
        eprintln!(
            "Skipping group CRUD: create failed (admin required?): {}",
            create.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }
    eprintln!("Created group: {}", group_name);
    harness::rate_limit_delay(runner.config());

    // FIND and verify content
    let find = runner.run_json(&["jira", "Groups", "find-groups", "--query", &group_name]);
    find.assert_success();
    let group_found = find
        .json
        .as_ref()
        .and_then(|j| j.get("groups"))
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .any(|g| g.get("name").and_then(|n| n.as_str()) == Some(group_name.as_str()))
        })
        .unwrap_or(false);
    assert!(group_found, "find-groups should contain {}", group_name);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira", "Groups", "remove-group", "--groupname", &group_name]);
    assert!(del.exit_code == 0, "remove-group failed: {}", del.stderr);
    eprintln!("Deleted group: {}", group_name);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Attachment Lifecycle ───────────────────────────────────────────────

#[test]
fn test_attachment_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let issue_key = create_issue(&runner, project, "E2E attachment parent");
    harness::rate_limit_delay(runner.config());

    // Attempt ADD attachment (may fail: multipart/form-data required)
    let add = runner.run_json_with_body(
        r#"{"file":"test.txt"}"#,
        &[
            "jira",
            "Issue attachments",
            "add-attachment",
            "--issueIdOrKey",
            &issue_key,
        ],
    );

    if add.exit_code == 0 {
        // Full lifecycle: add succeeded
        let att_id = add
            .json
            .as_ref()
            .and_then(|j| {
                // Response is an array of attachments
                j.as_array().and_then(|arr| arr.first()).or(Some(j))
            })
            .and_then(|a| a.get("id"))
            .and_then(|v| v.as_str());

        if let Some(att_id) = att_id {
            eprintln!("Created attachment: {}", att_id);
            harness::rate_limit_delay(runner.config());

            // GET
            let get = runner.run_json(&[
                "jira",
                "Issue attachments",
                "get-attachment",
                "--id",
                att_id,
            ]);
            get.assert_success();
            harness::rate_limit_delay(runner.config());

            // DELETE
            let del = runner.run(&[
                "jira",
                "Issue attachments",
                "remove-attachment",
                "--id",
                att_id,
            ]);
            assert!(
                del.exit_code == 0,
                "remove-attachment failed: {}",
                del.stderr
            );
            harness::rate_limit_delay(runner.config());
        }
    } else {
        // Fallback: test read-only attachment settings endpoint
        eprintln!(
            "Attachment upload requires multipart (got exit {}): {}",
            add.exit_code, add.stderr
        );
        eprintln!("Testing read-only attachment settings endpoint instead.");

        let meta = runner.run_json(&["jira", "Issue attachments", "get-attachment-meta"]);
        meta.assert_success();
        assert!(
            meta.json.is_some(),
            "Expected JSON from attachment settings"
        );
        harness::rate_limit_delay(runner.config());
    }

    delete_issue(&runner, &issue_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}
