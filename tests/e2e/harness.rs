//! E2E test harness: environment config, CLI runner, skip guard, rate limiter.

use std::env;
use std::thread;
use std::time::Duration;

/// Configuration for E2E tests, read from environment variables.
#[derive(Clone)]
#[allow(dead_code)]
pub struct E2eConfig {
    pub site: String,
    pub email: String,
    pub token: String,
    pub jira_project: String,
    pub confluence_space: String,
    pub delay_ms: u64,
    pub timeout_secs: u64,
}

impl E2eConfig {
    /// Read E2E configuration from environment variables.
    /// Returns None if required variables (SITE, EMAIL, TOKEN) are missing.
    pub fn from_env() -> Option<Self> {
        let site = env::var("SHRUG_E2E_SITE").ok()?;
        let email = env::var("SHRUG_E2E_EMAIL").ok()?;
        let token = env::var("SHRUG_E2E_TOKEN").ok()?;

        let jira_project =
            env::var("SHRUG_E2E_JIRA_PROJECT").unwrap_or_else(|_| "TEST".to_string());
        let confluence_space =
            env::var("SHRUG_E2E_CONFLUENCE_SPACE").unwrap_or_else(|_| "TEST".to_string());
        let delay_ms = env::var("SHRUG_E2E_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(200);
        let timeout_secs = env::var("SHRUG_E2E_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        Some(Self {
            site,
            email,
            token,
            jira_project,
            confluence_space,
            delay_ms,
            timeout_secs,
        })
    }
}

/// Skip the current test if E2E environment variables are not set.
/// Returns the E2eConfig if available, otherwise returns early from the test.
macro_rules! skip_unless_e2e {
    () => {
        match $crate::harness::E2eConfig::from_env() {
            Some(c) => c,
            None => {
                eprintln!(
                    "Skipping: E2E env vars not set \
                     (need SHRUG_E2E_SITE, SHRUG_E2E_EMAIL, SHRUG_E2E_TOKEN)"
                );
                return;
            }
        }
    };
}

/// Result of running the shrug CLI binary.
pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub json: Option<serde_json::Value>,
}

impl RunResult {
    /// Panic with full diagnostics if exit code is not 0.
    pub fn assert_success(&self) {
        assert_eq!(
            self.exit_code, 0,
            "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
            self.exit_code, self.stdout, self.stderr
        );
    }

    /// Panic with full diagnostics if exit code does not match expected.
    #[allow(dead_code)]
    pub fn assert_exit_code(&self, expected: i32) {
        assert_eq!(
            self.exit_code, expected,
            "Expected exit code {}, got {}.\nstdout: {}\nstderr: {}",
            expected, self.exit_code, self.stdout, self.stderr
        );
    }

    /// Panic with full stdout if the expected string is not found.
    pub fn assert_stdout_contains(&self, s: &str) {
        assert!(
            self.stdout.contains(s),
            "Expected stdout to contain {:?}, but it was:\n{}",
            s, self.stdout
        );
    }

    /// Look up a field in the parsed JSON using JSON pointer syntax (e.g., "/key/0/name").
    #[allow(dead_code)]
    pub fn json_field(&self, pointer: &str) -> Option<&serde_json::Value> {
        self.json.as_ref()?.pointer(pointer)
    }
}

/// CLI runner that executes the shrug binary with E2E credentials.
pub struct ShrugRunner {
    config: E2eConfig,
}

impl ShrugRunner {
    pub fn new(config: E2eConfig) -> Self {
        Self { config }
    }

    /// Execute shrug with the given arguments and capture output.
    pub fn run(&self, args: &[&str]) -> RunResult {
        let output = assert_cmd::Command::cargo_bin("shrug")
            .expect("Failed to find shrug binary")
            .args(args)
            .env("SHRUG_SITE", &self.config.site)
            .env("SHRUG_EMAIL", &self.config.email)
            .env("SHRUG_API_TOKEN", &self.config.token)
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .output()
            .expect("Failed to execute shrug binary");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        RunResult {
            stdout,
            stderr,
            exit_code,
            json: None,
        }
    }

    /// Execute shrug with --output json prepended, parsing the response.
    pub fn run_json(&self, args: &[&str]) -> RunResult {
        let mut all_args: Vec<&str> = vec!["--output", "json"];
        all_args.extend_from_slice(args);

        let mut result = self.run(&all_args);
        result.json = serde_json::from_str(&result.stdout).ok();
        result
    }

    /// Access the underlying config.
    pub fn config(&self) -> &E2eConfig {
        &self.config
    }
}

/// Sleep between API calls to avoid hitting Atlassian rate limits.
pub fn rate_limit_delay(config: &E2eConfig) {
    thread::sleep(Duration::from_millis(config.delay_ms));
}
