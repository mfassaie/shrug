//! Test helpers for mocked E2E tests.
//!
//! Provides a `MockEnv` that spins up an httpmock server, creates a CLI profile
//! pointing at that server, and builds CLI commands that use the profile with
//! env-var-injected credentials for authentication.
//!
//! Profile creation and deletion use the real `shrug profile` commands rather
//! than writing files directly. This is cross-platform (the `directories` crate
//! uses Windows API calls on Windows, not environment variables).

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use assert_cmd::Command;
use httpmock::prelude::*;

/// Global counter for unique profile names within a test run.
static PROFILE_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Unique profile name for each MockEnv instance, avoiding collisions
/// when tests run in parallel within the same binary.
/// Profile names must be lowercase alphanumeric + hyphens, starting with a letter/digit.
fn mock_profile_name() -> String {
    let n = PROFILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mocked-{}-{}", std::process::id(), n)
}

/// A self-contained mock test environment.
///
/// Spins up an httpmock server, creates a CLI profile pointing at that server,
/// and provides a method to build CLI commands pre-configured with env-var
/// credentials targeting the mock.
pub struct MockEnv {
    /// The httpmock server instance. Register mocks on this before calling `cmd()`.
    pub server: MockServer,
    /// Profile name created for this test.
    profile_name: String,
    /// Timeout for CLI commands.
    pub timeout_secs: u64,
}

impl MockEnv {
    /// Create a new mock environment with a running httpmock server and
    /// a fresh CLI profile.
    pub fn new() -> Self {
        let server = MockServer::start();
        let profile_name = mock_profile_name();

        let env = Self {
            server,
            profile_name,
            timeout_secs: 30,
        };

        env.create_profile();
        env
    }

    /// Build a `Command` for the shrug binary with env vars set to
    /// authenticate against the mock server using the test profile.
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("shrug").expect("Failed to find shrug binary");

        // Inject credentials via env vars (highest priority in credential resolution).
        // SHRUG_SITE overrides the profile's site, pointing at the mock server.
        cmd.env("SHRUG_API_TOKEN", "mock-api-token");
        cmd.env("SHRUG_EMAIL", "test@example.com");
        cmd.env("SHRUG_SITE", self.server.base_url());

        // Select the test profile
        cmd.env("SHRUG_PROFILE", &self.profile_name);

        cmd.timeout(Duration::from_secs(self.timeout_secs));

        cmd
    }

    /// Base URL of the mock server.
    #[allow(dead_code)]
    pub fn base_url(&self) -> String {
        self.server.base_url()
    }

    /// Create the test profile via `shrug profile create`.
    fn create_profile(&self) {
        let output = Command::cargo_bin("shrug")
            .expect("Failed to find shrug binary")
            .args(&[
                "profile", "create", &self.profile_name,
                "--site", &self.server.base_url(),
                "--email", "test@example.com",
            ])
            .timeout(Duration::from_secs(10))
            .output()
            .expect("Failed to run profile create");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success() || stderr.contains("already exists"),
            "Failed to create mock profile '{}': {}",
            self.profile_name,
            stderr
        );
    }

    /// Delete the test profile via `shrug profile delete`.
    fn delete_profile(&self) {
        let _ = Command::cargo_bin("shrug")
            .expect("Failed to find shrug binary")
            .args(&["profile", "delete", &self.profile_name])
            .timeout(Duration::from_secs(10))
            .output();
    }
}

impl Drop for MockEnv {
    fn drop(&mut self) {
        self.delete_profile();
    }
}

/// Assert that a command succeeded (exit code 0) and return (stdout, stderr).
pub fn assert_success(cmd: &mut Command) -> (String, String) {
    let output = cmd.output().expect("Failed to execute shrug binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);

    assert_eq!(
        code, 0,
        "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
        code, stdout, stderr
    );

    (stdout, stderr)
}

/// Assert that a command failed with a non-zero exit code and return (stdout, stderr, exit_code).
#[allow(dead_code)]
pub fn assert_failure(cmd: &mut Command) -> (String, String, i32) {
    let output = cmd.output().expect("Failed to execute shrug binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);

    assert_ne!(
        code, 0,
        "Expected non-zero exit code, got 0.\nstdout: {}\nstderr: {}",
        stdout, stderr
    );

    (stdout, stderr, code)
}

/// Parse stdout as JSON, panicking with diagnostics on failure.
pub fn parse_json(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout).unwrap_or_else(|e| {
        panic!(
            "Failed to parse stdout as JSON: {}\nstdout was:\n{}",
            e, stdout
        )
    })
}
