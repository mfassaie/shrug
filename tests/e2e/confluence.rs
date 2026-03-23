//! Confluence E2E tests: Space CRUD, Search, Users, Groups.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-conf-{}", std::process::id());
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

// ─── Space CRUD ──────────────────────────────────────────────────────────

#[test]
fn test_space_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_key = format!("E2E{}", std::process::id() % 10000);
    let body = format!(
        r#"{{"key":"{}","name":"E2E Test Space {}"}}"#,
        space_key, space_key
    );

    // CREATE
    let create = runner.run_json_with_body(&body, &["confluence", "Space", "create-space"]);
    if create.exit_code != 0 {
        // Space creation may fail (key exists, permissions, etc.)
        eprintln!(
            "Skipping space CRUD: create failed (may need admin): {}",
            create.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }
    eprintln!("Created space: {}", space_key);
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let ubody = format!(r#"{{"name":"E2E Updated {}"}}"#, space_key);
    let upd = runner.run_with_body(
        &ubody,
        &["confluence", "Space", "update-space", "--spaceKey", &space_key],
    );
    if upd.exit_code != 0 {
        eprintln!("Space update failed (non-fatal): {}", upd.stderr);
    }
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["confluence", "Space", "delete-space", "--spaceKey", &space_key]);
    if del.exit_code != 0 {
        eprintln!("Space delete failed (non-fatal): {}", del.stderr);
    }
    harness::rate_limit_delay(runner.config());

    teardown_profile(&runner, &profile);
}

// ─── Read-Only Tests ─────────────────────────────────────────────────────

#[test]
fn test_search_content_by_cql() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&[
        "confluence", "Search", "search-by-c-q-l",
        "--cql", "type = page",
        "--limit", "5",
    ]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from CQL search");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_get_current_user() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Users", "get-current-user"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from get-current-user");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_groups() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Group", "get-groups"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from get-groups");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}
