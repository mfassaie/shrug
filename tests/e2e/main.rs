//! E2E tests for shrug CLI against live Atlassian Cloud.
//!
//! These tests require environment variables to be set:
//! - SHRUG_E2E_SITE: Atlassian site URL (e.g., "mysite.atlassian.net")
//! - SHRUG_E2E_EMAIL: Atlassian account email
//! - SHRUG_E2E_TOKEN: Atlassian API token
//!
//! Optional:
//! - SHRUG_E2E_JIRA_PROJECT: Jira project key (default: "TEST")
//! - SHRUG_E2E_CONFLUENCE_SPACE: Confluence space key (default: "TEST")
//! - SHRUG_E2E_DELAY_MS: Inter-request delay in ms (default: 200)
//! - SHRUG_E2E_TIMEOUT_SECS: Command timeout in seconds (default: 30)
//!
//! Run with: cargo test --test e2e -- --test-threads=1

#[macro_use]
mod harness;
mod auth;
mod fixtures;
mod smoke;
