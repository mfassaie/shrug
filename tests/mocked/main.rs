//! Mocked E2E tests for shrug CLI.
//!
//! These tests run the full CLI binary against httpmock servers, verifying
//! that commands construct correct HTTP requests and handle responses properly.
//! No real Atlassian Cloud credentials are needed.
//!
//! Run with: cargo test --test mocked
//!
//! # Coverage gap analysis (Phase 12 reference)
//!
//! The static CLI exposes 37 entities across 3 products with ~140 commands total.
//! Current mocked tests cover **6 entities** (25 tests). The table below lists
//! every entity and its mocked test status.
//!
//! ## Jira (7 entities, ~50 commands)
//!
//! | Entity                      | Verbs covered           | Status    |
//! |-----------------------------|-------------------------|-----------|
//! | jira issue                  | list, create, view,     | PARTIAL   |
//! |                             | edit, delete, dry-run   |           |
//! | jira issue comment          | --                      | MISSING   |
//! | jira issue worklog          | --                      | MISSING   |
//! | jira issue attachment       | --                      | MISSING   |
//! | jira issue link             | --                      | MISSING   |
//! | jira issue watcher          | --                      | MISSING   |
//! | jira issue property         | --                      | MISSING   |
//! | jira issue remote-link      | --                      | MISSING   |
//! | jira project                | list, create, view      | PARTIAL   |
//! | jira project component      | --                      | MISSING   |
//! | jira project version        | --                      | MISSING   |
//! | jira filter                 | --                      | MISSING   |
//! | jira dashboard              | --                      | MISSING   |
//! | jira label                  | --                      | MISSING   |
//! | jira audit                  | --                      | MISSING   |
//! | jira search                 | --                      | MISSING   |
//!
//! ## Jira Software (3 entities, ~12 commands)
//!
//! | Entity                      | Verbs covered           | Status    |
//! |-----------------------------|-------------------------|-----------|
//! | jira-software board         | list, create, view,     | COVERED   |
//! |                             | delete                  |           |
//! | jira-software sprint        | list, create, edit,     | COVERED   |
//! |                             | delete                  |           |
//! | jira-software epic          | --                      | MISSING   |
//!
//! ## Confluence (10 entities, ~65 commands)
//!
//! | Entity                      | Verbs covered           | Status    |
//! |-----------------------------|-------------------------|-----------|
//! | confluence space             | list, create, view     | PARTIAL   |
//! | confluence space property    | --                     | MISSING   |
//! | confluence page              | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence page comment      | --                     | MISSING   |
//! | confluence page label        | --                     | MISSING   |
//! | confluence page like         | --                     | MISSING   |
//! | confluence page property     | --                     | MISSING   |
//! | confluence page attachment   | --                     | MISSING   |
//! | confluence page version      | --                     | MISSING   |
//! | confluence page restriction  | --                     | MISSING   |
//! | confluence blogpost          | --                     | MISSING   |
//! | confluence whiteboard        | --                     | MISSING   |
//! | confluence database          | --                     | MISSING   |
//! | confluence folder            | --                     | MISSING   |
//! | confluence custom-content    | --                     | MISSING   |
//! | confluence smart-link        | --                     | MISSING   |
//! | confluence task              | --                     | MISSING   |
//! | confluence search            | --                     | MISSING   |
//!
//! ## Summary
//!
//! - **Covered (full LCRUD):** 3 entities (jsw board, jsw sprint, confluence page)
//! - **Partial:** 3 entities (jira issue, jira project, confluence space)
//! - **Missing:** 31 entities
//! - **Priority for Phase 12:** jira issue sub-entities (comment, worklog, link,
//!   watcher, attachment, property, remote-link), jira filter, jira dashboard,
//!   jira search, confluence blogpost, confluence page sub-entities, jsw epic.

mod helpers;
mod jira_issue;
mod jira_project;
mod jsw_board;
mod jsw_sprint;
mod confluence_space;
mod confluence_page;
