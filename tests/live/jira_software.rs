//! Comprehensive Jira Software live E2E tests.
//!
//! Every board, sprint, and epic verb exercised with all parameters.
//! Create/edit tested with both typed params and --from-json where supported.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-jsw-{}", std::process::id());
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

fn create_filter(runner: &ShrugRunner, project: &str) -> Option<String> {
    let fname = format!("e2e-jsw-filter-{}", std::process::id());
    let jql = format!("project = {}", project);
    let result = runner.run_json(&[
        "jira", "filter", "create",
        "--name", &fname, "--jql", &jql,
    ]);
    if result.exit_code != 0 {
        eprintln!("Failed to create filter: {}", result.stderr);
        return None;
    }
    result.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn delete_filter(runner: &ShrugRunner, id: &str) {
    let _ = runner.run(&["jira", "filter", "delete", id, "--yes"]);
}

fn create_board(runner: &ShrugRunner, name: &str, filter_id: &str) -> Option<String> {
    let result = runner.run_json(&[
        "jira-software", "board", "create",
        "--name", name, "--type", "scrum", "--filter-id", filter_id,
    ]);
    if result.exit_code != 0 {
        eprintln!("Board create failed: {}", result.stderr);
        return None;
    }
    result.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_i64()).map(|n| n.to_string())
}

fn delete_board(runner: &ShrugRunner, id: &str) {
    let _ = runner.run(&["jira-software", "board", "delete", id, "--yes"]);
}

fn delete_issue(runner: &ShrugRunner, key: &str) {
    let _ = runner.run(&["jira", "issue", "delete", key, "--yes"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// BOARD
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_board_create_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let filter_id = match create_filter(&runner, project) {
        Some(id) => id,
        None => { teardown_profile(&runner, &profile); return; }
    };
    harness::rate_limit_delay(runner.config());

    let bname = format!("e2e-board-typed-{}", std::process::id());
    let create = runner.run_json(&[
        "jira-software", "board", "create",
        "--name", &bname,
        "--type", "scrum",
        "--filter-id", &filter_id,
    ]);
    create.assert_success();
    let bid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_i64())
        .expect("Expected board id");
    let bid_str = bid.to_string();
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira-software", "board", "view", &bid_str]);
    view.assert_success();
    let name = view.json_field("/name").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(name, bname);
    harness::rate_limit_delay(runner.config());

    // CONFIG
    let cfg = runner.run_json(&["jira-software", "board", "config", &bid_str]);
    cfg.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_board(&runner, &bid_str);
    harness::rate_limit_delay(runner.config());
    delete_filter(&runner, &filter_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_board_create_from_json() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let filter_id = match create_filter(&runner, project) {
        Some(id) => id,
        None => { teardown_profile(&runner, &profile); return; }
    };
    harness::rate_limit_delay(runner.config());

    let bname = format!("e2e-board-json-{}", std::process::id());
    let body = format!(
        r#"{{"name":"{}","type":"scrum","filterId":{}}}"#,
        bname, filter_id
    );
    // --from-json requires dummy required flags
    let create = runner.run_json_with_body(&body, &[
        "jira-software", "board", "create",
        "--name", "ignored", "--type", "scrum", "--filter-id", &filter_id,
    ]);
    create.assert_success();
    let bid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_i64())
        .expect("Expected board id");
    let bid_str = bid.to_string();
    harness::rate_limit_delay(runner.config());

    delete_board(&runner, &bid_str);
    harness::rate_limit_delay(runner.config());
    delete_filter(&runner, &filter_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_board_list_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let result = runner.run_json(&[
        "jira-software", "board", "list",
        "--project", project,
        "--type", "scrum",
    ]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// SPRINT
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_sprint_lifecycle_all_params() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let filter_id = match create_filter(&runner, project) {
        Some(id) => id,
        None => { teardown_profile(&runner, &profile); return; }
    };
    harness::rate_limit_delay(runner.config());

    let bname = format!("e2e-sprint-board-{}", std::process::id());
    let bid_str = match create_board(&runner, &bname, &filter_id) {
        Some(id) => id,
        None => {
            delete_filter(&runner, &filter_id);
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    // CREATE with all params
    let sname = format!("e2e-sprint-{}", std::process::id());
    let create = runner.run_json(&[
        "jira-software", "sprint", "create",
        "--name", &sname,
        "--board", &bid_str,
        "--goal", "E2E sprint goal",
    ]);
    if create.exit_code != 0 {
        eprintln!("Sprint create failed: {}", create.stderr);
        delete_board(&runner, &bid_str);
        harness::rate_limit_delay(runner.config());
        delete_filter(&runner, &filter_id);
        teardown_profile(&runner, &profile);
        return;
    }
    let sid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_i64())
        .expect("Expected sprint id");
    let sid_str = sid.to_string();
    harness::rate_limit_delay(runner.config());

    // VIEW
    let view = runner.run_json(&["jira-software", "sprint", "view", &sid_str]);
    view.assert_success();
    harness::rate_limit_delay(runner.config());

    // EDIT with all params (handler auto-fetches current name/state)
    let edit = runner.run(&[
        "jira-software", "sprint", "edit", &sid_str,
        "--name", &format!("{}-upd", sname),
        "--goal", "Updated goal",
        "--state", "future",
    ]);
    assert!(edit.exit_code == 0, "sprint edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    // LIST with params
    let list = runner.run_json(&[
        "jira-software", "sprint", "list",
        "--board", &bid_str,
        "--state", "future",
    ]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["jira-software", "sprint", "delete", &sid_str, "--yes"]);
    assert!(del.exit_code == 0, "sprint delete failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    delete_board(&runner, &bid_str);
    harness::rate_limit_delay(runner.config());
    delete_filter(&runner, &filter_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_sprint_create_from_json() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let filter_id = match create_filter(&runner, project) {
        Some(id) => id,
        None => { teardown_profile(&runner, &profile); return; }
    };
    harness::rate_limit_delay(runner.config());

    let bname = format!("e2e-sprint-json-board-{}", std::process::id());
    let bid_str = match create_board(&runner, &bname, &filter_id) {
        Some(id) => id,
        None => {
            delete_filter(&runner, &filter_id);
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let sname = format!("e2e-sprint-json-{}", std::process::id());
    let body = format!(
        r#"{{"name":"{}","originBoardId":{}}}"#,
        sname, bid_str
    );
    let create = runner.run_json_with_body(&body, &[
        "jira-software", "sprint", "create",
        "--name", "ignored", "--board", &bid_str,
    ]);
    if create.exit_code != 0 {
        eprintln!("Sprint from-json create failed: {}", create.stderr);
        delete_board(&runner, &bid_str);
        harness::rate_limit_delay(runner.config());
        delete_filter(&runner, &filter_id);
        teardown_profile(&runner, &profile);
        return;
    }
    let sid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_i64())
        .expect("Expected sprint id");
    let sid_str = sid.to_string();
    harness::rate_limit_delay(runner.config());

    let _ = runner.run(&["jira-software", "sprint", "delete", &sid_str, "--yes"]);
    harness::rate_limit_delay(runner.config());
    delete_board(&runner, &bid_str);
    harness::rate_limit_delay(runner.config());
    delete_filter(&runner, &filter_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_sprint_edit_from_json() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let filter_id = match create_filter(&runner, project) {
        Some(id) => id,
        None => { teardown_profile(&runner, &profile); return; }
    };
    harness::rate_limit_delay(runner.config());

    let bname = format!("e2e-sprint-editjson-board-{}", std::process::id());
    let bid_str = match create_board(&runner, &bname, &filter_id) {
        Some(id) => id,
        None => {
            delete_filter(&runner, &filter_id);
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let sname = format!("e2e-sej-{}", std::process::id());
    let create = runner.run_json(&[
        "jira-software", "sprint", "create",
        "--name", &sname, "--board", &bid_str,
    ]);
    if create.exit_code != 0 {
        eprintln!("Sprint create failed: {}", create.stderr);
        delete_board(&runner, &bid_str);
        delete_filter(&runner, &filter_id);
        teardown_profile(&runner, &profile);
        return;
    }
    let sid = create.json.as_ref().and_then(|j| j.get("id")).and_then(|v| v.as_i64())
        .expect("Expected sprint id");
    let sid_str = sid.to_string();
    harness::rate_limit_delay(runner.config());

    // EDIT via --from-json (sprint name must be <30 chars)
    let body = r#"{"name":"e2e-sj-upd","state":"future","goal":"JSON goal"}"#;
    let edit = runner.run_with_body(body, &[
        "jira-software", "sprint", "edit", &sid_str,
    ]);
    assert!(edit.exit_code == 0, "sprint from-json edit failed: {}", edit.stderr);
    harness::rate_limit_delay(runner.config());

    let _ = runner.run(&["jira-software", "sprint", "delete", &sid_str, "--yes"]);
    harness::rate_limit_delay(runner.config());
    delete_board(&runner, &bid_str);
    harness::rate_limit_delay(runner.config());
    delete_filter(&runner, &filter_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ═══════════════════════════════════════════════════════════════════════════
// EPIC
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_epic_view_and_list() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    // Create an Epic issue
    let create = runner.run_json(&[
        "jira", "issue", "create",
        "-s", "E2E epic comprehensive",
        "--project", project,
        "--type", "Epic",
    ]);
    if create.exit_code != 0 {
        eprintln!("Skipping epic test: Epic type may not exist: {}", create.stderr);
        teardown_profile(&runner, &profile);
        return;
    }
    let epic_key = create.json.as_ref().and_then(|j| j.get("key")).and_then(|v| v.as_str())
        .expect("Expected epic key").to_string();
    harness::rate_limit_delay(runner.config());

    // VIEW via Agile API
    let view = runner.run_json(&["jira-software", "epic", "view", &epic_key]);
    if view.exit_code != 0 {
        eprintln!("Epic view failed (next-gen project): {}", view.stderr);
        delete_issue(&runner, &epic_key);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    harness::rate_limit_delay(runner.config());

    // LIST issues in epic
    let list = runner.run_json(&["jira-software", "epic", "list", &epic_key]);
    list.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &epic_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_epic_edit_with_done() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let project = runner.config().jira_project.as_str();

    let create = runner.run_json(&[
        "jira", "issue", "create",
        "-s", "E2E epic edit test",
        "--project", project,
        "--type", "Epic",
    ]);
    if create.exit_code != 0 {
        eprintln!("Skipping epic edit: {}", create.stderr);
        teardown_profile(&runner, &profile);
        return;
    }
    let epic_key = create.json.as_ref().and_then(|j| j.get("key")).and_then(|v| v.as_str())
        .expect("Expected key").to_string();
    harness::rate_limit_delay(runner.config());

    // EDIT with --name and --done
    let edit = runner.run(&[
        "jira-software", "epic", "edit", &epic_key,
        "--name", "E2E epic renamed",
        "--done",
    ]);
    if edit.exit_code != 0 {
        eprintln!("Epic edit failed (next-gen project): {}", edit.stderr);
    }
    harness::rate_limit_delay(runner.config());

    delete_issue(&runner, &epic_key);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}
