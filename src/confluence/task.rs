//! Confluence task entity: list, view, edit operations (v2 API).
//!
//! Tasks cannot be created or deleted directly via the API.

use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::auth::credentials::ResolvedCredential;
use crate::cli::{ColorChoice, OutputFormat};
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Task entity subcommands.
#[derive(Subcommand)]
pub enum TaskCommands {
    /// List tasks
    List {
        /// Filter by space ID
        #[arg(long)]
        space_id: Option<String>,
        /// Filter by page ID
        #[arg(long)]
        page_id: Option<String>,
        /// Filter by status (complete, incomplete)
        #[arg(long)]
        status: Option<String>,
        /// Filter by assignee
        #[arg(short = 'a', long)]
        assignee: Option<String>,
    },
    /// View a task
    View {
        /// Task ID
        id: String,
    },
    /// Edit a task (change status)
    Edit {
        /// Task ID
        id: String,
        /// New status (complete, incomplete)
        status: String,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for task edit.
pub fn build_edit_body(id: &str, status: &str) -> Value {
    json!({
        "id": id,
        "status": status,
    })
}

/// Build query parameters for task list.
pub fn build_list_query_params(
    space_id: Option<&str>,
    page_id: Option<&str>,
    status: Option<&str>,
    assignee: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(s) = space_id {
        params.push(("space-id".to_string(), s.to_string()));
    }
    if let Some(p) = page_id {
        params.push(("page-id".to_string(), p.to_string()));
    }
    if let Some(st) = status {
        params.push(("status".to_string(), st.to_string()));
    }
    if let Some(a) = assignee {
        params.push(("assignee".to_string(), a.to_string()));
    }
    params
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a task command.
pub fn execute(
    cmd: &TaskCommands,
    credential: &ResolvedCredential,
    client: &Client,
    output_format: &OutputFormat,
    color: &ColorChoice,
    limit: Option<u32>,
    dry_run: bool,
) -> Result<(), ShrugError> {
    let base_url = http::build_base_url(credential);
    let color_enabled = match color {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => {
            std::env::var("NO_COLOR").is_err()
                && is_terminal::is_terminal(std::io::stdout())
        }
    };

    match cmd {
        TaskCommands::List {
            space_id,
            page_id,
            status,
            assignee,
        } => {
            let query_params = build_list_query_params(
                space_id.as_deref(),
                page_id.as_deref(),
                status.as_deref(),
                assignee.as_deref(),
            );
            let url_base = http::build_url(
                &base_url, "/wiki/api/v2/tasks", &HashMap::new(), &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url_base, None);
                return Ok(());
            }

            let results = http::execute_paginated_get(
                client, &url_base, credential, &query_params, &[], limit, 25, true,
            )?;
            if !results.is_empty() {
                let json_val = if matches!(output_format, OutputFormat::Json) {
                    serde_json::Value::Array(results)
                } else {
                    output::project_array(&results, &[
                        ("ID", "/id"),
                        ("Status", "/status"),
                        ("Body", "/body"),
                    ])
                };
                let formatted = output::format_response(
                    &json_val.to_string(), output_format,
                    is_terminal::is_terminal(std::io::stdout()), color_enabled, None,
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        TaskCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/tasks/{id}",
                &path_params,
                &[],
            );

            let result = http::execute_request(
                client,
                Method::GET,
                &url,
                Some(credential),
                None,
                &[],
            )?;

            if let Some(ref json_val) = result {
                let display_val = if matches!(output_format, OutputFormat::Json) {
                    json_val.clone()
                } else {
                    output::project(json_val, &[
                        ("ID", "/id"),
                        ("Status", "/status"),
                        ("Body", "/body"),
                        ("Created By", "/createdBy"),
                        ("Created", "/createdAt"),
                    ])
                };
                let formatted = output::format_response(
                    &display_val.to_string(),
                    output_format,
                    is_terminal::is_terminal(std::io::stdout()),
                    color_enabled,
                    None,
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        TaskCommands::Edit { id, status } => {
            let request_body = build_edit_body(id, status);

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/tasks/{id}",
                &path_params,
                &[],
            );

            http::execute_request(
                client,
                Method::PUT,
                &url,
                Some(credential),
                Some(&request_body),
                &[],
            )?;

            match output_format {
                OutputFormat::Json => {
                    println!("{}", json!({"id": id, "status": status}));
                }
                _ => println!("Updated task {} to {}", id, status),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_edit_body() {
        let body = build_edit_body("12345", "complete");
        assert_eq!(body["id"], "12345");
        assert_eq!(body["status"], "complete");
    }

    #[test]
    fn test_task_list_url() {
        let query_params = build_list_query_params(
            Some("12345"),
            Some("67890"),
            Some("incomplete"),
            None,
        );
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/tasks",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/wiki/api/v2/tasks"));
        assert!(url.contains("space-id=12345"));
        assert!(url.contains("page-id=67890"));
        assert!(url.contains("status=incomplete"));
    }

    #[test]
    fn test_task_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "55555".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/tasks/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/tasks/55555"
        );
    }
}
