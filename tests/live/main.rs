//! Live E2E tests for shrug CLI against real Atlassian Cloud.
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
//! Run with: cargo test --test live -- --test-threads=1
//!
//! # Gap analysis for Phase 13 rewrite
//!
//! ALL live tests use the **old dynamic CLI command syntax** (capitalised tag
//! names + operationId-based subcommands). The static CLI uses lowercase entity
//! names with standard LCRUD verbs. Every test that calls the API needs rewriting.
//!
//! ## Command pattern comparison
//!
//! | Old (dynamic)                                      | New (static)                          |
//! |----------------------------------------------------|---------------------------------------|
//! | `jira Issues create-issue`                         | `jira issue create`                   |
//! | `jira Issues get-issue --issueIdOrKey X`           | `jira issue view X`                   |
//! | `jira Issues edit-issue --issueIdOrKey X`          | `jira issue edit X`                   |
//! | `jira Issues delete-issue --issueIdOrKey X`        | `jira issue delete X`                 |
//! | `jira Issue search search-and-reconsile-...`       | `jira search list --jql ...`          |
//! | `jira Issue comments add-comment`                  | `jira issue comment create`           |
//! | `jira Issue worklogs add-worklog`                  | `jira issue worklog create`           |
//! | `jira Filters create-filter`                       | `jira filter create`                  |
//! | `jira Dashboards create-dashboard`                 | `jira dashboard create`               |
//! | `jira Projects search-projects`                    | `jira project list`                   |
//! | `jira Project versions create-version`             | `jira project version create`         |
//! | `jira Project components create-component`         | `jira project component create`       |
//! | `jira Issue types get-issue-all-types`             | `jira label list` (or similar)        |
//! | `jira Groups create-group`                         | (no static entity yet)                |
//! | `confluence Space get-spaces`                      | `confluence space list`               |
//! | `confluence Page create-page`                      | `confluence page create`              |
//! | `confluence Blog Post create-blog-post`            | `confluence blogpost create`          |
//! | `jira-software Board create-board`                 | `jira-software board create`          |
//! | `jira-software Sprint create-sprint`               | `jira-software sprint create`         |
//!
//! ## File-by-file stale status
//!
//! | File              | Tests | Stale? | Notes                                        |
//! |-------------------|-------|--------|----------------------------------------------|
//! | smoke.rs          | 4     | ALL    | Uses dynamic commands for help/version/API   |
//! | auth.rs           | 6     | MOST   | Profile tests mostly OK, but API calls use   |
//! |                   |       |        | dynamic `Issue search` syntax                |
//! | jira.rs           | 15    | ALL    | Every test uses dynamic `Issues`, `Filters`, |
//! |                   |       |        | `Dashboards`, `Project versions` etc.        |
//! | confluence.rs     | 15    | ALL    | Every test uses dynamic `Space`, `Page`,     |
//! |                   |       |        | `Blog Post`, `Folder` etc.                   |
//! | jira_software.rs  | 9     | ALL    | Every test uses dynamic `Board`, `Sprint`,   |
//! |                   |       |        | `Epic`, `Issue`, `Backlog` syntax            |
//! | features.rs       | 11    | ALL    | Output format, dry-run, pagination tests all |
//! |                   |       |        | use dynamic `Issue search` syntax            |
//! | fixtures.rs       | 0     | STALE  | Helper uses very old `"+create"`, `"issues"`,|
//! |                   |       |        | `"deleteIssue"` syntax (pre-dynamic era)     |
//!
//! ## Entities tested (dynamic) vs static CLI coverage needed
//!
//! Currently tested entities (all via old syntax):
//! - Jira: issue CRUD, comment CRUD, worklog CRUD, filter CRUD, dashboard CRUD,
//!   version CRUD, component CRUD, issue types (list), fields (list),
//!   statuses (list), priorities (list), resolutions (list), watchers,
//!   votes, issue links, groups, attachments, CRUD verb aliases
//! - Confluence: page CRUD, space (list), blog post CRUD, space properties CRUD,
//!   content properties CRUD, folder CRUD, comments (list), versions (list),
//!   likes (list), tasks (list), attachments (list), custom content (list),
//!   ancestors (list), descendants (list), space roles (list), whiteboard CRUD
//! - JSW: board CRUD, sprint lifecycle, board list, epic ops, JSW issue get,
//!   board get-issues, sprint move-issues, backlog move
//!
//! Missing from live tests entirely:
//! - Jira: search (static entity), label (static entity), audit (static entity),
//!   issue property, issue remote-link
//! - Confluence: page label, page restriction, database, smart-link,
//!   confluence search
//! - JSW: (good coverage, minor gaps in epic edit)
//!
//! ## Harness changes needed for Phase 13
//!
//! 1. **ShrugRunner.run_json_with_body()** passes `--json` as a global flag.
//!    Static CLI entities use typed flags (`-s`, `--project`, `--body`) instead
//!    of raw JSON. The `run_json_with_body` helper will still work for the few
//!    commands that accept `--from-json`, but most tests should switch to typed
//!    CLI flags.
//!
//! 2. **Helper functions** (create_issue, delete_issue, create_page, etc.) in
//!    each test file use dynamic syntax and raw JSON bodies. All need rewriting
//!    to use static CLI commands with typed flags.
//!
//! 3. **fixtures.rs** `ResourceTracker` cleanup commands use very old syntax
//!    (`"issues"`, `"deleteIssue"`). Needs full rewrite, or better, each test
//!    file should use its own cleanup helpers (as jira.rs and confluence.rs
//!    already do).
//!
//! 4. **Profile boilerplate**: every test file duplicates `setup_profile` /
//!    `teardown_profile`. These should be consolidated into the harness.

#[macro_use]
mod harness;
mod auth;
mod confluence;
mod features;
mod fixtures;
mod jira;
mod jira_software;
mod smoke;
