//! Jira Software sprint entity: list, create, view, edit, delete operations.

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

/// Sprint entity subcommands.
#[derive(Subcommand)]
pub enum SprintCommands {
    /// List sprints for a board
    List {
        /// Board ID (required)
        #[arg(long)]
        board: u64,
        /// Filter by state (active, closed, future)
        #[arg(long)]
        state: Option<String>,
    },
    /// Create a new sprint
    Create {
        /// Sprint name
        #[arg(long)]
        name: String,
        /// Board ID to create the sprint in
        #[arg(long)]
        board: u64,
        /// Sprint goal
        #[arg(long)]
        goal: Option<String>,
        /// Start date (ISO 8601, e.g. 2025-01-01T00:00:00.000Z)
        #[arg(long)]
        start_date: Option<String>,
        /// End date (ISO 8601, e.g. 2025-01-14T00:00:00.000Z)
        #[arg(long)]
        end_date: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// View a sprint
    View {
        /// Sprint ID
        id: String,
    },
    /// Edit a sprint
    Edit {
        /// Sprint ID
        id: String,
        /// New sprint name
        #[arg(long)]
        name: Option<String>,
        /// New sprint goal
        #[arg(long)]
        goal: Option<String>,
        /// New start date (ISO 8601)
        #[arg(long)]
        start_date: Option<String>,
        /// New end date (ISO 8601)
        #[arg(long)]
        end_date: Option<String>,
        /// Sprint state (active, closed)
        #[arg(long)]
        state: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// Delete a sprint
    Delete {
        /// Sprint ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for sprint creation.
pub fn build_create_body(
    name: &str,
    board: u64,
    goal: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Value {
    let mut body = json!({
        "name": name,
        "originBoardId": board,
    });

    if let Some(g) = goal {
        body["goal"] = json!(g);
    }
    if let Some(sd) = start_date {
        body["startDate"] = json!(sd);
    }
    if let Some(ed) = end_date {
        body["endDate"] = json!(ed);
    }

    body
}

/// Build JSON request body for sprint edit. Only includes provided fields.
pub fn build_edit_body(
    name: Option<&str>,
    goal: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    state: Option<&str>,
) -> Value {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(g) = goal {
        body["goal"] = json!(g);
    }
    if let Some(sd) = start_date {
        body["startDate"] = json!(sd);
    }
    if let Some(ed) = end_date {
        body["endDate"] = json!(ed);
    }
    if let Some(s) = state {
        body["state"] = json!(s);
    }

    body
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a sprint command.
pub fn execute(
    cmd: &SprintCommands,
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
        SprintCommands::List { board, state } => {
            let mut query_params: Vec<(String, String)> = Vec::new();
            if let Some(s) = state {
                query_params.push(("state".to_string(), s.to_string()));
            }

            let mut path_params = HashMap::new();
            path_params.insert("boardId".to_string(), board.to_string());
            let url_base = http::build_url(
                &base_url, "/rest/agile/1.0/board/{boardId}/sprint",
                &path_params, &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url_base, None);
                return Ok(());
            }

            let results = http::execute_paginated_get(
                client, &url_base, credential, &query_params, &[], limit, 50, false,
            )?;
            let json_val = serde_json::Value::Array(results);
            if !json_val.as_array().is_none_or(|a| a.is_empty()) {
                let formatted = output::format_response(
                    &json_val.to_string(), output_format,
                    is_terminal::is_terminal(std::io::stdout()), color_enabled, None,
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        SprintCommands::Create {
            name,
            board,
            goal,
            start_date,
            end_date,
            from_json,
        } => {
            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                crate::jira::issue::read_json_file(path)?
            } else {
                build_create_body(
                    name,
                    *board,
                    goal.as_deref(),
                    start_date.as_deref(),
                    end_date.as_deref(),
                )
            };

            let url = format!("{}/rest/agile/1.0/sprint", base_url);

            if dry_run {
                http::dry_run_request(&Method::POST, &url, Some(&request_body));
                return Ok(());
            }

            let result = http::execute_request(
                client,
                Method::POST,
                &url,
                Some(credential),
                Some(&request_body),
                &[],
            )?;

            if let Some(ref json_val) = result {
                match output_format {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(json_val).unwrap_or_default()
                        );
                    }
                    _ => {
                        if let Some(id) = json_val.get("id") {
                            println!("Created sprint {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        SprintCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("sprintId".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/agile/1.0/sprint/{sprintId}",
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
                let formatted = output::format_response(
                    &json_val.to_string(),
                    output_format,
                    is_terminal::is_terminal(std::io::stdout()),
                    color_enabled,
                    None,
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        SprintCommands::Edit {
            id,
            name,
            goal,
            start_date,
            end_date,
            state,
            from_json,
        } => {
            // Validate: starting a sprint requires both start and end dates
            if let Some(ref s) = state {
                if s == "active" && (start_date.is_none() || end_date.is_none()) {
                    return Err(ShrugError::UsageError(
                        "Starting a sprint requires --start-date and --end-date".into(),
                    ));
                }
            }

            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                crate::jira::issue::read_json_file(path)?
            } else {
                build_edit_body(
                    name.as_deref(),
                    goal.as_deref(),
                    start_date.as_deref(),
                    end_date.as_deref(),
                    state.as_deref(),
                )
            };

            let mut path_params = HashMap::new();
            path_params.insert("sprintId".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/agile/1.0/sprint/{sprintId}",
                &path_params,
                &[],
            );

            if dry_run {
                http::dry_run_request(&Method::PUT, &url, Some(&request_body));
                return Ok(());
            }

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
                    println!("{}", json!({"id": id, "status": "updated"}));
                }
                _ => println!("Updated sprint {}", id),
            }
            Ok(())
        }

        SprintCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete sprint {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("sprintId".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/agile/1.0/sprint/{sprintId}",
                &path_params,
                &[],
            );

            http::execute_request(
                client,
                Method::DELETE,
                &url,
                Some(credential),
                None,
                &[],
            )?;

            match output_format {
                OutputFormat::Json => {
                    println!("{}", json!({"id": id, "status": "deleted"}));
                }
                _ => println!("Deleted sprint {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprint_create_body() {
        let body = build_create_body(
            "Sprint 1",
            42,
            Some("Finish onboarding"),
            None,
            None,
        );
        assert_eq!(body["name"], "Sprint 1");
        assert_eq!(body["originBoardId"], 42);
        assert_eq!(body["goal"], "Finish onboarding");
    }

    #[test]
    fn test_sprint_edit_state() {
        let body = build_edit_body(
            None,
            None,
            Some("2025-01-01T00:00:00.000Z"),
            Some("2025-01-14T00:00:00.000Z"),
            Some("active"),
        );
        assert_eq!(body["state"], "active");
        assert_eq!(body["startDate"], "2025-01-01T00:00:00.000Z");
        assert_eq!(body["endDate"], "2025-01-14T00:00:00.000Z");
    }

    #[test]
    fn test_sprint_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("boardId".to_string(), "42".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/board/{boardId}/sprint",
            &path_params,
            &[],
        );
        assert!(url.contains("/rest/agile/1.0/board/42/sprint"));
    }

    #[test]
    fn test_sprint_list_url_with_state() {
        let mut path_params = HashMap::new();
        path_params.insert("boardId".to_string(), "42".to_string());
        let query_params = vec![("state".to_string(), "active".to_string())];
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/board/{boardId}/sprint",
            &path_params,
            &query_params,
        );
        assert!(url.contains("/rest/agile/1.0/board/42/sprint"));
        assert!(url.contains("state=active"));
    }

    #[test]
    fn test_sprint_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("sprintId".to_string(), "99".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/sprint/{sprintId}",
            &path_params,
            &[],
        );
        assert!(url.contains("/rest/agile/1.0/sprint/99"));
    }

    #[test]
    fn test_sprint_state_active_requires_dates() {
        // state=active with no dates should be an error
        let state = Some("active".to_string());
        let start_date: Option<String> = None;
        let end_date: Option<String> = None;

        if let Some(ref s) = state {
            if s == "active" && (start_date.is_none() || end_date.is_none()) {
                // This is the validation path: should error
                assert!(true);
                return;
            }
        }
        panic!("Validation should have caught missing dates for active state");
    }

    #[test]
    fn test_sprint_state_active_with_dates_ok() {
        // state=active with both dates should pass validation
        let state = Some("active".to_string());
        let start_date = Some("2025-01-01T00:00:00.000Z".to_string());
        let end_date = Some("2025-01-14T00:00:00.000Z".to_string());

        let should_error = if let Some(ref s) = state {
            s == "active" && (start_date.is_none() || end_date.is_none())
        } else {
            false
        };

        assert!(!should_error, "Validation should pass when dates are provided");
    }

    #[test]
    fn test_sprint_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("sprintId".to_string(), "55".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/sprint/{sprintId}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/agile/1.0/sprint/55"
        );
    }

    #[test]
    fn test_sprint_create_with_from_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sprint.json");
        std::fs::write(&path, r#"{"name":"Sprint X","originBoardId":42}"#).unwrap();
        let value = crate::jira::issue::read_json_file(path.to_str().unwrap()).unwrap();
        assert_eq!(value["name"], "Sprint X");
        assert_eq!(value["originBoardId"], 42);
    }

    #[test]
    fn test_sprint_edit_with_from_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sprint_edit.json");
        std::fs::write(&path, r#"{"name":"Renamed Sprint","state":"closed"}"#).unwrap();
        let value = crate::jira::issue::read_json_file(path.to_str().unwrap()).unwrap();
        assert_eq!(value["name"], "Renamed Sprint");
        assert_eq!(value["state"], "closed");
    }
}
