//! Smoke tests for shrug CLI against an installed binary on PATH.
//!
//! Unlike the E2E tests (tests/e2e/) which use `cargo_bin("shrug")` to test the
//! build output, these tests run against an installed shrug.exe found on PATH
//! (or via the SHRUG_E2E_BINARY environment variable).
//!
//! **Offline tests** (no credentials needed):
//!   Run with: cargo test --test smoke
//!   Requires: shrug on PATH or SHRUG_E2E_BINARY set
//!
//! **Online tests** (credentials required):
//!   Run with: cargo test --test smoke -- --test-threads=1
//!   Requires: shrug on PATH + SHRUG_E2E_SITE, SHRUG_E2E_EMAIL, SHRUG_E2E_TOKEN
//!
//! Optional:
//! - SHRUG_E2E_BINARY: Absolute path to shrug binary (overrides PATH lookup)
//! - SHRUG_E2E_JIRA_PROJECT: Jira project key (default: "TEST")
//! - SHRUG_E2E_CONFLUENCE_SPACE: Confluence space key (default: "TEST")
//! - SHRUG_E2E_DELAY_MS: Inter-request delay in ms (default: 200)
//! - SHRUG_E2E_TIMEOUT_SECS: Command timeout in seconds (default: 30)

#[macro_use]
mod harness;
mod error_messages;
mod fixtures;
mod global_flags;
mod help_messages;
mod live_api;
mod static_commands;
mod validation;
