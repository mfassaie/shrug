//! Static command smoke tests — offline tests for profile, auth, cache, and completions.
//!
//! All tests in this module run without Atlassian credentials.
//! They exercise local CLI commands against the installed shrug binary.

use crate::harness::SmokeRunner;

/// Generate a unique profile name with module-scoped prefix to avoid collisions.
fn unique_name(prefix: &str) -> String {
    format!("sc-{}-{}", prefix, std::process::id())
}

/// Create a test profile with dummy credentials. Caller must clean up.
fn create_test_profile(runner: &SmokeRunner, name: &str) {
    let result = runner.run(&[
        "profile",
        "create",
        name,
        "--site",
        "test.atlassian.net",
        "--email",
        "test@example.com",
    ]);
    assert!(
        result.exit_code == 0 || result.stderr.contains("already exists"),
        "Failed to create profile '{}': {}",
        name,
        result.stderr
    );
}

/// Delete a test profile (best-effort, no panic on failure).
fn delete_test_profile(runner: &SmokeRunner, name: &str) {
    let _ = runner.run(&["profile", "delete", name]);
}

// ─── Profile Lifecycle Tests ─────────────────────────────────────────────

#[test]
fn test_profile_create_and_list() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-create");

    create_test_profile(&runner, &name);

    let list = runner.run(&["profile", "list"]);
    list.assert_success();
    list.assert_stdout_contains(&name);

    delete_test_profile(&runner, &name);
}

#[test]
fn test_profile_get_details() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-get");

    create_test_profile(&runner, &name);

    let result = runner.run(&["profile", "get", &name]);
    result.assert_success();
    result.assert_stdout_contains(&name);
    result.assert_stdout_contains("test.atlassian.net");
    result.assert_stdout_contains("test@example.com");

    delete_test_profile(&runner, &name);
}

#[test]
fn test_profile_delete_removes() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("smoke-del");

    create_test_profile(&runner, &name);

    let del = runner.run(&["profile", "delete", &name]);
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

// ─── Auth Tests ──────────────────────────────────────────────────────────

#[test]
fn test_auth_help() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["auth", "--help"]);
    result.assert_success();
    // Help should mention auth subcommands
    assert!(
        result.stdout.contains("set-token") || result.stdout.contains("status"),
        "auth --help should mention subcommands.\nstdout: {}",
        result.stdout
    );
}

#[test]
fn test_auth_status_with_profile() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);
    let name = unique_name("auth-status");

    create_test_profile(&runner, &name);

    // Use --profile flag to target the specific test profile
    let result = runner.run(&["auth", "status", "--profile", &name]);
    // Auth status should work with a profile that has no stored token
    // It may exit non-zero (no token) but should not crash
    if result.exit_code == 0 {
        result.assert_stdout_contains(&name);
    }

    delete_test_profile(&runner, &name);
}

// ─── Cache Tests ─────────────────────────────────────────────────────────

#[test]
fn test_cache_help() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["cache", "--help"]);
    result.assert_success();
    result.assert_stdout_contains("refresh");
}

// Shell completions: no CLI subcommand exposed. Covered by 11 unit tests
// in src/completions.rs (bash, zsh, fish, powershell — static + dynamic).

// ─── Dynamic Completions ────────────────────────────────────────────────

#[test]
fn test_dynamic_complete_subcommand_exists() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    // _complete is a hidden subcommand for shell tab-completion
    // It needs a valid completion type. "projects" needs auth so may fail,
    // but the subcommand itself should be recognised (not "unrecognized subcommand").
    let result = runner.run(&["_complete", "projects"]);
    // May fail with auth error (exit 3) but should NOT fail with usage error (exit 2)
    assert!(
        result.exit_code != 2,
        "_complete subcommand should be recognised (got exit 2): {}",
        result.stderr
    );
}
