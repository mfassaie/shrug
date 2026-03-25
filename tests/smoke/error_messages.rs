//! Error message validation tests — verify error output format and exit codes.
//!
//! Tests trigger error conditions that are reproducible offline (no API needed)
//! and validate that errors follow the "Error: message\nHint: remediation" pattern.

use crate::harness::SmokeRunner;

// ─── Profile Error Tests ─────────────────────────────────────────────────

#[test]
fn test_error_nonexistent_profile_show() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["profile", "view", "nonexistent-xyz-999"]);
    assert_ne!(
        result.exit_code, 0,
        "Showing nonexistent profile should fail"
    );
    assert!(
        result.stderr.contains("Error") || result.stderr.contains("error"),
        "Error output should contain 'Error'.\nstderr: {}",
        result.stderr
    );
    assert!(
        result.stderr.contains("Hint") || result.stderr.contains("hint"),
        "Error output should contain remediation hint.\nstderr: {}",
        result.stderr
    );
}

#[test]
fn test_error_nonexistent_profile_delete() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["profile", "delete", "nonexistent-xyz-999"]);
    assert_ne!(
        result.exit_code, 0,
        "Deleting nonexistent profile should fail"
    );
    assert!(
        result.stderr.contains("Error") || result.stderr.contains("error"),
        "Error output should contain 'Error'.\nstderr: {}",
        result.stderr
    );
}

// ─── Usage Error Tests ───────────────────────────────────────────────────

#[test]
fn test_error_invalid_output_format() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--output", "invalid-format", "--help"]);
    assert_ne!(
        result.exit_code, 0,
        "Invalid --output format should fail.\nstdout: {}\nstderr: {}",
        result.stdout, result.stderr
    );
}

#[test]
fn test_error_invalid_color_mode() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--color", "rainbow", "--help"]);
    assert_ne!(
        result.exit_code, 0,
        "Invalid --color value should fail.\nstdout: {}\nstderr: {}",
        result.stdout, result.stderr
    );
}

#[test]
fn test_error_missing_required_arg() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    // profile create without required --name, --site, --email
    let result = runner.run(&["profile", "create"]);
    assert_ne!(
        result.exit_code, 0,
        "profile create without args should fail"
    );
    assert!(
        result.stderr.contains("required") || result.stderr.contains("<NAME>"),
        "Error should mention required arguments.\nstderr: {}",
        result.stderr
    );
}

// ─── Auth Error Tests ────────────────────────────────────────────────────

#[test]
fn test_error_auth_status_graceful() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    // auth status with a nonexistent profile — should fail gracefully
    let result = runner.run(&["auth", "status", "--profile", "nonexistent-xyz-999"]);
    // May fail with auth or profile error, but must not crash
    if result.exit_code != 0 {
        assert!(
            !result.stderr.is_empty(),
            "Auth status error should produce stderr output"
        );
    }
}

// ─── Error Format Consistency ────────────────────────────────────────────

#[test]
fn test_error_hint_present_on_profile_error() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["profile", "view", "nonexistent-xyz-999"]);
    assert_ne!(result.exit_code, 0);
    // Verify the error follows the "Error: ...\nHint: ..." pattern
    assert!(
        result.stderr.contains("Error"),
        "Should contain 'Error' prefix.\nstderr: {}",
        result.stderr
    );
    assert!(
        result.stderr.contains("Hint"),
        "Should contain remediation 'Hint'.\nstderr: {}",
        result.stderr
    );
}

#[test]
fn test_exit_codes_nonzero_on_error() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    // Multiple error scenarios should all exit non-zero
    let scenarios: Vec<(&[&str], &str)> = vec![
        (
            &["profile", "view", "nonexistent"],
            "nonexistent profile view",
        ),
        (&["--output", "bad", "--help"], "invalid output format"),
        (&["--color", "bad", "--help"], "invalid color mode"),
        (&["profile", "create"], "missing required args"),
    ];

    for (args, description) in scenarios {
        let result = runner.run(args);
        assert_ne!(
            result.exit_code, 0,
            "'{}' should exit non-zero, got 0.\nstdout: {}\nstderr: {}",
            description, result.stdout, result.stderr
        );
    }
}
