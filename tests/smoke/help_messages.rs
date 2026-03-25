//! Help message validation tests — verify help output structure and content.
//!
//! Uses insta for golden-file snapshots on key help outputs, plus
//! assertion-based tests for structural validation.

use crate::harness::SmokeRunner;

// ─── Snapshot Tests ──────────────────────────────────────────────────────

#[test]
fn test_help_snapshot_toplevel() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--help"]);
    result.assert_success();
    insta::assert_snapshot!("toplevel_help", result.stdout);
}

#[test]
fn test_help_snapshot_auth() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["auth", "--help"]);
    result.assert_success();
    insta::assert_snapshot!("auth_help", result.stdout);
}

#[test]
fn test_help_snapshot_profile() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["profile", "--help"]);
    result.assert_success();
    insta::assert_snapshot!("profile_help", result.stdout);
}

// ─── Structure Validation Tests ──────────────────────────────────────────

#[test]
fn test_help_toplevel_lists_products() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--help"]);
    result.assert_success();

    let stdout = &result.stdout;
    assert!(stdout.contains("jira"), "Help should list jira product");
    assert!(
        stdout.contains("confluence"),
        "Help should list confluence product"
    );
}

#[test]
fn test_help_toplevel_lists_commands() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--help"]);
    result.assert_success();

    let stdout = &result.stdout;
    assert!(stdout.contains("auth"), "Help should list auth command");
    assert!(
        stdout.contains("profile"),
        "Help should list profile command"
    );
}

#[test]
fn test_help_toplevel_lists_global_flags() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--help"]);
    result.assert_success();

    let stdout = &result.stdout;
    assert!(
        stdout.contains("--output"),
        "Help should list --output flag"
    );
    assert!(stdout.contains("--color"), "Help should list --color flag");
    assert!(
        stdout.contains("--verbose"),
        "Help should list --verbose flag"
    );
    assert!(
        stdout.contains("--dry-run"),
        "Help should list --dry-run flag"
    );
}

#[test]
fn test_help_profile_lists_subcommands() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["profile", "--help"]);
    result.assert_success();

    let stdout = &result.stdout;
    assert!(stdout.contains("create"), "Profile help should list create");
    assert!(stdout.contains("list"), "Profile help should list list");
    assert!(stdout.contains("show"), "Profile help should list show");
    assert!(stdout.contains("delete"), "Profile help should list delete");
    assert!(stdout.contains("use"), "Profile help should list use");
}

#[test]
fn test_help_auth_lists_subcommands() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["auth", "--help"]);
    result.assert_success();

    let stdout = &result.stdout;
    assert!(
        stdout.contains("set-token"),
        "Auth help should list set-token"
    );
    assert!(stdout.contains("status"), "Auth help should list status");
    assert!(stdout.contains("login"), "Auth help should list login");
    assert!(stdout.contains("setup"), "Auth help should list setup");
}

#[test]
fn test_version_format() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--version"]);
    result.assert_success();

    let version = result.stdout.trim();
    // Version should match "shrug X.Y.Z" pattern
    assert!(
        version.starts_with("shrug "),
        "Version should start with 'shrug ', got: {}",
        version
    );
    let ver_str = version.strip_prefix("shrug ").unwrap();
    let base = ver_str.split('-').next().unwrap();
    let parts: Vec<&str> = base.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "Version base should have 3 parts (X.Y.Z), got: {}",
        version
    );
}
