//! Smoke test harness: binary discovery, CLI runner, skip guards, rate limiter.

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// Configuration for smoke tests — binary path and timeout.
#[derive(Clone)]
pub struct SmokeConfig {
    pub binary_path: PathBuf,
    pub timeout_secs: u64,
}

impl SmokeConfig {
    /// Try to resolve the shrug binary path.
    ///
    /// 1. Checks SHRUG_E2E_BINARY env var (explicit path)
    /// 2. Falls back to PATH lookup via `which`
    /// 3. Returns None if neither works (does not panic)
    pub fn try_resolve() -> Option<Self> {
        let timeout_secs = env::var("SHRUG_E2E_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        // Check explicit binary path first
        if let Ok(path) = env::var("SHRUG_E2E_BINARY") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(Self {
                    binary_path: path,
                    timeout_secs,
                });
            }
            eprintln!(
                "Warning: SHRUG_E2E_BINARY set to {:?} but file does not exist",
                path
            );
            return None;
        }

        // Fall back to PATH lookup
        match which::which("shrug") {
            Ok(path) => Some(Self {
                binary_path: path,
                timeout_secs,
            }),
            Err(_) => None,
        }
    }

    /// Resolve the shrug binary path, panicking if not found.
    ///
    /// Use this only in contexts where binary absence is a hard failure.
    /// For skip macros, use try_resolve() instead.
    #[allow(dead_code)]
    pub fn resolve() -> Self {
        Self::try_resolve()
            .expect("shrug not found on PATH. Set SHRUG_E2E_BINARY or install shrug.")
    }
}

/// Configuration for online E2E tests, read from environment variables.
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

#[allow(dead_code)]
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
            s,
            self.stdout
        );
    }

    /// Panic with full stderr if the expected string is not found.
    #[allow(dead_code)]
    pub fn assert_stderr_contains(&self, s: &str) {
        assert!(
            self.stderr.contains(s),
            "Expected stderr to contain {:?}, but it was:\n{}",
            s,
            self.stderr
        );
    }

    /// Look up a field in the parsed JSON using JSON pointer syntax (e.g., "/key/0/name").
    #[allow(dead_code)]
    pub fn json_field(&self, pointer: &str) -> Option<&serde_json::Value> {
        self.json.as_ref()?.pointer(pointer)
    }
}

/// CLI runner that executes the installed shrug binary.
pub struct SmokeRunner {
    config: SmokeConfig,
    e2e: Option<E2eConfig>,
}

impl SmokeRunner {
    /// Create an offline runner (no credentials injected).
    /// Credential env vars are actively removed from child process to prevent leakage.
    pub fn new(config: SmokeConfig) -> Self {
        Self { config, e2e: None }
    }

    /// Create an online runner that injects E2E credentials.
    #[allow(dead_code)]
    pub fn with_e2e(config: SmokeConfig, e2e: E2eConfig) -> Self {
        Self {
            config,
            e2e: Some(e2e),
        }
    }

    /// Execute shrug with the given arguments and capture output.
    pub fn run(&self, args: &[&str]) -> RunResult {
        let mut cmd = assert_cmd::Command::new(&self.config.binary_path);
        cmd.args(args);
        cmd.timeout(Duration::from_secs(self.config.timeout_secs));

        match &self.e2e {
            Some(e2e) => {
                // Online mode: inject credentials
                cmd.env("SHRUG_SITE", &e2e.site);
                cmd.env("SHRUG_EMAIL", &e2e.email);
                cmd.env("SHRUG_API_TOKEN", &e2e.token);
            }
            None => {
                // Offline mode: remove credential env vars to prevent leakage
                cmd.env_remove("SHRUG_SITE");
                cmd.env_remove("SHRUG_EMAIL");
                cmd.env_remove("SHRUG_API_TOKEN");
            }
        }

        let output = cmd.output().expect("Failed to execute shrug binary");

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
    #[allow(dead_code)]
    pub fn run_json(&self, args: &[&str]) -> RunResult {
        let mut all_args: Vec<&str> = vec!["--output", "json"];
        all_args.extend_from_slice(args);

        let mut result = self.run(&all_args);
        result.json = serde_json::from_str(&result.stdout).ok();
        result
    }

    /// Execute shrug with --json body prepended (global flag) and --output json, parsing response.
    #[allow(dead_code)]
    pub fn run_json_with_body(&self, body: &str, args: &[&str]) -> RunResult {
        let mut all_args: Vec<&str> = vec!["--output", "json", "--json", body];
        all_args.extend_from_slice(args);

        let mut result = self.run(&all_args);
        result.json = serde_json::from_str(&result.stdout).ok();
        result
    }

    /// Execute shrug with --json body prepended, no JSON output parsing.
    #[allow(dead_code)]
    pub fn run_with_body(&self, body: &str, args: &[&str]) -> RunResult {
        let mut all_args: Vec<&str> = vec!["--json", body];
        all_args.extend_from_slice(args);
        self.run(&all_args)
    }

    /// Access the underlying config.
    #[allow(dead_code)]
    pub fn config(&self) -> &SmokeConfig {
        &self.config
    }

    /// Access the E2E config, if available.
    #[allow(dead_code)]
    pub fn e2e_config(&self) -> Option<&E2eConfig> {
        self.e2e.as_ref()
    }
}

/// Skip the current test if shrug binary is not found on PATH.
/// Returns SmokeConfig if binary found, otherwise returns early from the test.
macro_rules! skip_unless_binary {
    () => {
        match $crate::harness::SmokeConfig::try_resolve() {
            Some(c) => c,
            None => {
                eprintln!(
                    "Skipping: shrug binary not found on PATH \
                     (set SHRUG_E2E_BINARY or install shrug)"
                );
                return;
            }
        }
    };
}

/// Skip the current test if shrug binary is not found or E2E credentials are missing.
/// Returns (SmokeConfig, E2eConfig) tuple if both available.
#[allow(unused_macros)]
macro_rules! skip_unless_e2e {
    () => {{
        let smoke_config = match $crate::harness::SmokeConfig::try_resolve() {
            Some(c) => c,
            None => {
                eprintln!(
                    "Skipping: shrug binary not found on PATH \
                     (set SHRUG_E2E_BINARY or install shrug)"
                );
                return;
            }
        };
        let e2e_config = match $crate::harness::E2eConfig::from_env() {
            Some(c) => c,
            None => {
                eprintln!(
                    "Skipping: E2E env vars not set \
                     (need SHRUG_E2E_SITE, SHRUG_E2E_EMAIL, SHRUG_E2E_TOKEN)"
                );
                return;
            }
        };
        (smoke_config, e2e_config)
    }};
}

/// Sleep between API calls to avoid hitting Atlassian rate limits.
#[allow(dead_code)]
pub fn rate_limit_delay(config: &E2eConfig) {
    thread::sleep(Duration::from_millis(config.delay_ms));
}
