//! Mocked E2E tests for shrug CLI.
//!
//! These tests run the full CLI binary against httpmock servers, verifying
//! that commands construct correct HTTP requests and handle responses properly.
//! No real Atlassian Cloud credentials are needed.
//!
//! Run with: cargo test --test mocked

mod helpers;
mod jira_issue;
mod jira_project;
mod jsw_board;
mod jsw_sprint;
mod confluence_space;
mod confluence_page;
