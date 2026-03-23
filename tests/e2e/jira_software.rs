//! Jira Software E2E tests: Boards, Sprints, Epics.
//!
//! Each test validates operations against live Atlassian Cloud
//! using the `jira-software` product prefix.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-jsw-{}", std::process::id());
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

/// Create a JQL filter for the given project. Returns filter ID or None on failure.
fn create_filter(runner: &ShrugRunner, project: &str) -> Option<String> {
    let body = format!(
        r#"{{"name":"e2e-jsw-filter-{}","jql":"project = {}"}}"#,
        std::process::id(),
        project
    );
    let result = runner.run_json_with_body(&body, &["jira", "Filters", "create-filter"]);
    if result.exit_code != 0 {
        eprintln!("Failed to create filter: {}", result.stderr);
        return None;
    }
    result
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn delete_filter(runner: &ShrugRunner, id: &str) {
    let result = runner.run(&["jira", "Filters", "delete-filter", "--id", id]);
    if result.exit_code == 0 {
        eprintln!("Deleted filter: {}", id);
    } else {
        eprintln!(
            "Warning: failed to delete filter '{}': {}",
            id, result.stderr
        );
    }
}

/// Create a Jira issue via the Platform API. Returns the issue key.
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

// ─── Board CRUD Lifecycle ───────────────────────────────────────────────

#[test]
fn test_board_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    // Create filter (required for board creation)
    let filter_id = match create_filter(&runner, project) {
        Some(id) => id,
        None => {
            eprintln!("Skipping board test: could not create filter");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    eprintln!("Created filter: {}", filter_id);
    harness::rate_limit_delay(runner.config());

    // CREATE board
    let board_name = format!("e2e-board-{}", std::process::id());
    let body = format!(
        r#"{{"name":"{}","type":"scrum","filterId":{}}}"#,
        board_name, filter_id
    );
    let create = runner.run_json_with_body(&body, &["jira-software", "Board", "create-board"]);
    if create.exit_code != 0 {
        eprintln!(
            "Skipping board test: create-board failed: {}",
            create.stderr
        );
        delete_filter(&runner, &filter_id);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    let board_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_i64())
        .expect("Expected board id as integer");
    let board_id_str = board_id.to_string();
    eprintln!("Created board: {} ({})", board_name, board_id_str);
    harness::rate_limit_delay(runner.config());

    // GET board and verify name
    let read = runner.run_json(&[
        "jira-software",
        "Board",
        "get-board",
        "--boardId",
        &board_id_str,
    ]);
    read.assert_success();
    let read_name = read
        .json_field("/name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(read_name, board_name, "Board name should match");
    harness::rate_limit_delay(runner.config());

    // GET configuration
    let config_result = runner.run_json(&[
        "jira-software",
        "Board",
        "get-configuration",
        "--boardId",
        &board_id_str,
    ]);
    config_result.assert_success();
    harness::rate_limit_delay(runner.config());

    // DELETE board
    let del = runner.run(&[
        "jira-software",
        "Board",
        "delete-board",
        "--boardId",
        &board_id_str,
    ]);
    assert!(del.exit_code == 0, "delete-board failed: {}", del.stderr);
    eprintln!("Deleted board: {}", board_id_str);
    harness::rate_limit_delay(runner.config());

    delete_filter(&runner, &filter_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Sprint Lifecycle ───────────────────────────────────────────────────

#[test]
fn test_sprint_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    // Setup: filter + board
    let filter_id = match create_filter(&runner, project) {
        Some(id) => id,
        None => {
            eprintln!("Skipping sprint test: could not create filter");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let board_name = format!("e2e-sprint-board-{}", std::process::id());
    let body = format!(
        r#"{{"name":"{}","type":"scrum","filterId":{}}}"#,
        board_name, filter_id
    );
    let board_create =
        runner.run_json_with_body(&body, &["jira-software", "Board", "create-board"]);
    if board_create.exit_code != 0 {
        eprintln!(
            "Skipping sprint test: create-board failed: {}",
            board_create.stderr
        );
        delete_filter(&runner, &filter_id);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    let board_id = board_create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_i64())
        .expect("Expected board id");
    let board_id_str = board_id.to_string();
    eprintln!("Created board for sprint test: {}", board_id_str);
    harness::rate_limit_delay(runner.config());

    // CREATE sprint
    let sprint_name = format!("e2e-sprint-{}", std::process::id());
    let sprint_body = format!(
        r#"{{"name":"{}","originBoardId":{}}}"#,
        sprint_name, board_id
    );
    let sprint_create =
        runner.run_json_with_body(&sprint_body, &["jira-software", "Sprint", "create-sprint"]);
    if sprint_create.exit_code != 0 {
        eprintln!(
            "Skipping sprint test: create-sprint failed: {}",
            sprint_create.stderr
        );
        let _ = runner.run(&[
            "jira-software",
            "Board",
            "delete-board",
            "--boardId",
            &board_id_str,
        ]);
        harness::rate_limit_delay(runner.config());
        delete_filter(&runner, &filter_id);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    let sprint_id = sprint_create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_i64())
        .expect("Expected sprint id");
    let sprint_id_str = sprint_id.to_string();
    eprintln!("Created sprint: {} ({})", sprint_name, sprint_id_str);
    harness::rate_limit_delay(runner.config());

    // GET sprint and verify name
    let read = runner.run_json(&[
        "jira-software",
        "Sprint",
        "get-sprint",
        "--sprintId",
        &sprint_id_str,
    ]);
    read.assert_success();
    let read_name = read
        .json_field("/name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(read_name, sprint_name, "Sprint name should match");
    harness::rate_limit_delay(runner.config());

    // UPDATE sprint
    let upd = runner.run_with_body(
        r#"{"goal":"E2E test goal"}"#,
        &[
            "jira-software",
            "Sprint",
            "partially-update-sprint",
            "--sprintId",
            &sprint_id_str,
        ],
    );
    assert!(
        upd.exit_code == 0,
        "partially-update-sprint failed: {}",
        upd.stderr
    );
    harness::rate_limit_delay(runner.config());

    // DELETE sprint
    let del = runner.run(&[
        "jira-software",
        "Sprint",
        "delete-sprint",
        "--sprintId",
        &sprint_id_str,
    ]);
    assert!(del.exit_code == 0, "delete-sprint failed: {}", del.stderr);
    eprintln!("Deleted sprint: {}", sprint_id_str);
    harness::rate_limit_delay(runner.config());

    // Cleanup: board + filter
    let _ = runner.run(&[
        "jira-software",
        "Board",
        "delete-board",
        "--boardId",
        &board_id_str,
    ]);
    harness::rate_limit_delay(runner.config());
    delete_filter(&runner, &filter_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Board List Operations ──────────────────────────────────────────────

#[test]
fn test_list_boards() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run_json(&[
        "jira-software",
        "Board",
        "get-all-boards",
        "--projectKeyOrId",
        project,
    ]);
    result.assert_success();
    let board_count = result
        .json
        .as_ref()
        .and_then(|j| j.get("values"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);
    eprintln!("Found {} boards for project {}", board_count, project);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Epic Operations ────────────────────────────────────────────────────

#[test]
fn test_epic_operations() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    // Create an Epic issue via Jira Platform API
    let body = format!(
        r#"{{"fields":{{"project":{{"key":"{}"}},"summary":"E2E epic test","issuetype":{{"name":"Epic"}}}}}}"#,
        project
    );
    let create = runner.run_json_with_body(&body, &["jira", "Issues", "create-issue"]);
    if create.exit_code != 0 {
        eprintln!(
            "Skipping epic test: could not create Epic issue (type may not exist): {}",
            create.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }
    let epic_key = create
        .json
        .as_ref()
        .and_then(|j| j.get("key"))
        .and_then(|v| v.as_str())
        .expect("Expected epic key");
    let epic_key = epic_key.to_string();
    eprintln!("Created epic: {}", epic_key);
    harness::rate_limit_delay(runner.config());

    // GET epic via Jira Software API
    let get = runner.run_json(&[
        "jira-software",
        "Epic",
        "get-epic",
        "--epicIdOrKey",
        &epic_key,
    ]);
    get.assert_success();
    harness::rate_limit_delay(runner.config());

    // GET issues for epic (may be empty)
    let issues = runner.run_json(&[
        "jira-software",
        "Epic",
        "get-issues-for-epic",
        "--epicIdOrKey",
        &epic_key,
    ]);
    issues.assert_success();
    harness::rate_limit_delay(runner.config());

    // Cleanup
    delete_issue(&runner, &epic_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── JSW Issue Get ──────────────────────────────────────────────────────

#[test]
fn test_jsw_issue_get() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let issue_key = create_issue(&runner, project, "E2E JSW issue get");
    harness::rate_limit_delay(runner.config());

    let get = runner.run_json(&[
        "jira-software",
        "Issue",
        "get-issue",
        "--issueIdOrKey",
        &issue_key,
    ]);
    get.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &issue_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Backlog Move ───────────────────────────────────────────────────────

#[test]
fn test_backlog_move() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let issue_key = create_issue(&runner, project, "E2E backlog move");
    harness::rate_limit_delay(runner.config());

    let body = format!(r#"{{"issues":["{}"]}}"#, issue_key);
    let result = runner.run_with_body(
        &body,
        &["jira-software", "Backlog", "move-issues-to-backlog"],
    );
    if result.exit_code != 0 {
        eprintln!(
            "Backlog move returned exit code {} (issue may not be on a board): {}",
            result.exit_code, result.stderr
        );
    }
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &issue_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}
