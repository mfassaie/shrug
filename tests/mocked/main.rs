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
//! ## Jira (16 entities, ~50 commands)
//!
//! | Entity                      | Verbs covered           | Status    |
//! |-----------------------------|-------------------------|-----------|
//! | jira issue                  | list, create, view,     | COVERED   |
//! |                             | edit, delete, dry-run   |           |
//! | jira issue comment          | list, create, view,     | COVERED   |
//! |                             | edit, delete            |           |
//! | jira issue worklog          | list, create, view,     | COVERED   |
//! |                             | edit, delete            |           |
//! | jira issue attachment       | list, create, view,     | COVERED   |
//! |                             | delete                  |           |
//! | jira issue link             | list, create, view,     | COVERED   |
//! |                             | delete                  |           |
//! | jira issue watcher          | list, create, delete    | COVERED   |
//! | jira issue property         | list, view, edit,       | COVERED   |
//! |                             | delete                  |           |
//! | jira issue remote-link      | list, create, view,     | COVERED   |
//! |                             | edit, delete            |           |
//! | jira project                | list, create, view,     | COVERED   |
//! |                             | edit, delete            |           |
//! | jira project component      | list, create, view,     | COVERED   |
//! |                             | edit, delete            |           |
//! | jira project version        | list, create, view,     | COVERED   |
//! |                             | edit, delete            |           |
//! | jira filter                 | list, create, view,     | COVERED   |
//! |                             | edit, delete            |           |
//! | jira dashboard              | list, create, view,     | COVERED   |
//! |                             | edit, delete            |           |
//! | jira label                  | list                    | COVERED   |
//! | jira audit                  | list                    | COVERED   |
//! | jira search                 | list                    | COVERED   |
//!
//! ## Jira Software (3 entities, ~12 commands)
//!
//! | Entity                      | Verbs covered           | Status    |
//! |-----------------------------|-------------------------|-----------|
//! | jira-software board         | list, create, view,     | COVERED   |
//! |                             | delete, config          |           |
//! | jira-software sprint        | list, create, edit,     | COVERED   |
//! |                             | delete                  |           |
//! | jira-software epic          | view, edit, list        | COVERED   |
//!
//! ## Confluence (18 entities, ~65 commands)
//!
//! | Entity                      | Verbs covered           | Status    |
//! |-----------------------------|-------------------------|-----------|
//! | confluence space             | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence space property    | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence page              | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence page comment      | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence page label        | list, create, delete   | COVERED   |
//! | confluence page like         | view, list             | COVERED   |
//! | confluence page property     | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence page attachment   | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence page version      | list, view             | COVERED   |
//! | confluence page restriction  | view, edit, delete     | COVERED   |
//! | confluence blogpost          | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence whiteboard        | create, view, delete   | COVERED   |
//! | confluence database          | create, view, delete   | COVERED   |
//! | confluence folder            | create, view, delete   | COVERED   |
//! | confluence custom-content    | list, create, view,    | COVERED   |
//! |                              | edit, delete           |           |
//! | confluence smart-link        | create, view, delete   | COVERED   |
//! | confluence task              | list, view, edit       | COVERED   |
//! | confluence search            | list                   | COVERED   |
//!
//! ## Summary
//!
//! - **Covered:** 37/37 entities (16 Jira + 3 JSW + 18 Confluence)
//! - **Missing:** 0
//! - **Note:** Epic edit test skipped (production bug: --color flag collision).
//!   Blogpost sub-entity variants not tested separately (shared handlers with page).

mod helpers;
mod jira_issue;
mod jira_project;
mod jira_filter;
mod jira_dashboard;
mod jira_label;
mod jira_audit;
mod jira_search;
mod jira_issue_comment;
mod jira_issue_worklog;
mod jira_issue_attachment;
mod jira_issue_watcher;
mod jira_issue_link;
mod jira_issue_remote_link;
mod jira_issue_property;
mod jira_project_component;
mod jira_project_version;
mod jsw_board;
mod jsw_sprint;
mod jsw_epic;
mod confluence_space;
mod confluence_page;
mod confluence_blogpost;
mod confluence_whiteboard;
mod confluence_database;
mod confluence_folder;
mod confluence_custom_content;
mod confluence_smart_link;
mod confluence_task;
mod confluence_search;
mod confluence_space_property;
mod confluence_page_comment;
mod confluence_page_label;
mod confluence_page_like;
mod confluence_page_property;
mod confluence_page_attachment;
mod confluence_page_version;
mod confluence_page_restriction;
