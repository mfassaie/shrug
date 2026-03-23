//! Auth workflow E2E tests: profile CRUD, env var auth, first-run detection,
//! --profile flag override, and auth status reporting.

use crate::harness::{self, ShrugRunner};

/// Generate a unique profile name using the process ID to avoid collisions.
fn unique_name(prefix: &str) -> String {
    format!("{}-{}", prefix, std::process::id())
}

/// Helper: create a profile and return its name. Caller must clean up.
fn create_profile(runner: &ShrugRunner, name: &str) -> String {
    let result = runner.run(&[
        "profile",
        "create",
        "--name",
        name,
        "--site",
        runner.config().site.as_str(),
        "--email",
        runner.config().email.as_str(),
    ]);
    assert!(
        result.exit_code == 0 || result.stderr.contains("already exists"),
        "Failed to create profile '{}': {}",
        name,
        result.stderr
    );
    name.to_string()
}

/// Helper: delete a profile (best-effort, no panic on failure).
fn delete_profile(runner: &ShrugRunner, name: &str) {
    let result = runner.run(&["profile", "delete", "--name", name]);
    if result.exit_code != 0 {
        eprintln!(
            "Warning: failed to delete profile '{}': {}",
            name, result.stderr
        );
    }
}

// ─── Profile Lifecycle Tests (AC-1, AC-2, AC-3) ─────────────────────────

#[test]
fn test_profile_create_and_list() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let name = unique_name("e2e-create");

    // Create
    let result = runner.run(&[
        "profile",
        "create",
        "--name",
        &name,
        "--site",
        runner.config().site.as_str(),
        "--email",
        runner.config().email.as_str(),
    ]);
    result.assert_success();
    result.assert_stdout_contains("created");

    // List
    let list = runner.run(&["profile", "list"]);
    list.assert_success();
    list.assert_stdout_contains(&name);

    // Clean up
    delete_profile(&runner, &name);
}

#[test]
fn test_profile_show_details() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let name = unique_name("e2e-show");

    create_profile(&runner, &name);

    let result = runner.run(&["profile", "show", "--name", &name]);
    result.assert_success();
    result.assert_stdout_contains(&name);
    // Site may have trailing slash stripped by profile storage
    let site_check = runner.config().site.trim_end_matches('/');
    result.assert_stdout_contains(site_check);
    result.assert_stdout_contains(runner.config().email.as_str());

    delete_profile(&runner, &name);
}

#[test]
fn test_profile_use_sets_default() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let name_a = unique_name("e2e-use-a");
    let name_b = unique_name("e2e-use-b");

    create_profile(&runner, &name_a);
    create_profile(&runner, &name_b);

    let result = runner.run(&["profile", "use", "--name", &name_b]);
    result.assert_success();
    result.assert_stdout_contains("Now using");

    delete_profile(&runner, &name_a);
    delete_profile(&runner, &name_b);
}

#[test]
fn test_profile_delete() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let name = unique_name("e2e-del");

    create_profile(&runner, &name);

    let del = runner.run(&["profile", "delete", "--name", &name]);
    del.assert_success();

    let list = runner.run(&["profile", "list"]);
    list.assert_success();
    assert!(
        !list.stdout.contains(&name),
        "Deleted profile '{}' should not appear in list.\nstdout: {}",
        name,
        list.stdout
    );
}

// ─── Auth Workflow Tests (AC-4, AC-5, AC-6, AC-7, AC-8) ─────────────────

#[test]
fn test_env_var_auth_works() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let name = unique_name("e2e-envauth");

    create_profile(&runner, &name);
    let _ = runner.run(&["profile", "use", "--name", &name]);

    // Env vars (SHRUG_API_TOKEN, SHRUG_EMAIL, SHRUG_SITE) are set by ShrugRunner.
    // The profile exists, so credential resolution uses env var token.
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
        "Expected JSON response from live API.\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr
    );

    delete_profile(&runner, &name);
    harness::rate_limit_delay(runner.config());
}

#[test]
fn test_first_run_help_succeeds_without_auth() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    // Top-level help should always work, regardless of profile state.
    let result = runner.run(&["--help"]);
    result.assert_success();
    result.assert_stdout_contains("Atlassian");
}

#[test]
fn test_first_run_api_call_fails_gracefully() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);

    // Precondition: check if profiles already exist.
    let list = runner.run(&["profile", "list"]);
    let has_profiles = list.exit_code == 0
        && list
            .stdout
            .lines()
            .any(|line| !line.starts_with("NAME") && !line.starts_with('-') && !line.is_empty());

    if has_profiles {
        eprintln!(
            "Skipping test_first_run_api_call_fails_gracefully: \
             profiles already exist, first-run precondition not met"
        );
        return;
    }

    // No profiles — API call should fail with auth error.
    let result = runner.run(&[
        "jira",
        "Issue search",
        "search-and-reconsile-issues-using-jql",
        "--jql",
        "project = TEST",
        "--maxResults",
        "1",
    ]);
    result.assert_exit_code(3);
    assert!(
        result.stderr.contains("setup") || result.stderr.contains("profile"),
        "Auth error should mention 'setup' or 'profile'.\nstderr: {}",
        result.stderr
    );
}

#[test]
fn test_profile_flag_override() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let name_a = unique_name("e2e-prof-a");
    let name_b = unique_name("e2e-prof-b");

    create_profile(&runner, &name_a);
    create_profile(&runner, &name_b);
    let _ = runner.run(&["profile", "use", "--name", &name_a]);

    // Use --profile flag to override default
    let result = runner.run(&["--profile", &name_b, "profile", "show", "--name", &name_b]);
    result.assert_success();
    result.assert_stdout_contains(&name_b);

    delete_profile(&runner, &name_a);
    delete_profile(&runner, &name_b);
}

#[test]
fn test_auth_status_reports_for_profile() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let name = unique_name("e2e-status");

    create_profile(&runner, &name);

    let result = runner.run(&["auth", "status", "--profile", &name]);
    result.assert_success();
    result.assert_stdout_contains(&name);

    delete_profile(&runner, &name);
}
