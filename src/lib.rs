pub mod auth;
pub mod cli;         // exists as cli.rs now, refactored into cli/ directory in plan 04-02
pub mod cmd;         // still exists, will be stripped later
pub mod completions;
pub mod content;     // NEW: adf, markdown_to_adf, jql
pub mod core;        // NEW: config, error, exit_codes, logging, output, http, pagination, quirks
pub mod dynamic_completions;
pub mod executor;    // still exists, will be stripped later
pub mod jira;        // NEW: static Jira entity modules (Phase 5)
pub mod jsw;         // NEW: static JSW entity modules (Phase 6)
pub mod confluence;  // NEW: static Confluence entity modules (Phase 7)
pub mod spec;        // still exists, will be stripped later
