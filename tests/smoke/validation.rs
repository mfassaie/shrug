//! Validation tests to confirm the smoke test harness works.
//!
//! These are offline tests — they only need shrug on PATH, no credentials.

use crate::harness::SmokeRunner;

#[test]
fn test_version_output() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--version"]);
    result.assert_success();
    result.assert_stdout_contains("shrug");
}

#[test]
fn test_help_output() {
    let config = skip_unless_binary!();
    let runner = SmokeRunner::new(config);

    let result = runner.run(&["--help"]);
    result.assert_success();
    result.assert_stdout_contains("Atlassian");
}
