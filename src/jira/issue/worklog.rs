//! Jira issue worklog sub-entity: LCRUD operations.
//!
//! Supports timeSpent, started, comment (body), and --remaining for
//! adjusting the remaining estimate on create.

use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::auth::credentials::ResolvedCredential;
use crate::cli::OutputFormat;
use crate::content::markdown_to_adf;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Worklog subcommands.
#[derive(Subcommand)]
pub enum WorklogCommands {
    /// List worklogs on an issue
    List {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
    },
    /// Log work on an issue
    Create {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Time spent (e.g., 2h, 30m, 1d)
        #[arg(long)]
        time: String,
        /// Start date/time (ISO 8601, e.g., 2024-01-15T09:00:00.000+0000)
        #[arg(long)]
        started: Option<String>,
        /// Comment in markdown
        #[arg(short = 'b', long)]
        body: Option<String>,
        /// Set remaining estimate (e.g., 4h, 1d). Adds adjustEstimate=new to the request.
        #[arg(long)]
        remaining: Option<String>,
    },
    /// View a specific worklog entry
    View {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Worklog ID
        worklog_id: String,
    },
    /// Edit a worklog entry
    Edit {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Worklog ID
        worklog_id: String,
        /// Updated time spent
        #[arg(long)]
        time: Option<String>,
        /// Updated start date/time
        #[arg(long)]
        started: Option<String>,
        /// Updated comment in markdown
        #[arg(short = 'b', long)]
        body: Option<String>,
    },
    /// Delete a worklog entry
    Delete {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Worklog ID
        worklog_id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Build a worklog create request body.
pub fn build_create_body(
    time: &str,
    started: Option<&str>,
    body: Option<&str>,
) -> Value {
    let mut payload = json!({ "timeSpent": time });

    if let Some(s) = started {
        payload["started"] = json!(s);
    }
    if let Some(b) = body {
        payload["comment"] = markdown_to_adf::markdown_to_adf(b);
    }

    payload
}

/// Build a worklog edit request body. All fields are optional.
pub fn build_edit_body(
    time: Option<&str>,
    started: Option<&str>,
    body: Option<&str>,
) -> Value {
    let mut payload = json!({});

    if let Some(t) = time {
        payload["timeSpent"] = json!(t);
    }
    if let Some(s) = started {
        payload["started"] = json!(s);
    }
    if let Some(b) = body {
        payload["comment"] = markdown_to_adf::markdown_to_adf(b);
    }

    payload
}

/// Execute a worklog command.
pub fn execute(
    cmd: &WorklogCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        WorklogCommands::List { issue_key } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/worklog",
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
                    let items = json_val.get("worklogs")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    output::project_array(&items, &[
                        ("ID", "/id"),
                        ("Author", "/author"),
                        ("Time Spent", "/timeSpent"),
                        ("Started", "/started"),
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

        WorklogCommands::Create {
            issue_key,
            time,
            started,
            body,
            remaining,
        } => {
            let request_body = build_create_body(
                time,
                started.as_deref(),
                body.as_deref(),
            );

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());

            let query_params: Vec<(String, String)> = if let Some(ref est) = remaining {
                vec![
                    ("adjustEstimate".to_string(), "new".to_string()),
                    ("newEstimate".to_string(), est.clone()),
                ]
            } else {
                vec![]
            };

            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/worklog",
                &path_params,
                &query_params,
            );

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
                        if let Some(id) = json_val.get("id").and_then(|v| v.as_str()) {
                            println!("Created worklog {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        WorklogCommands::View {
            issue_key,
            worklog_id,
        } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("id".to_string(), worklog_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/worklog/{id}",
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
                        ("Author", "/author"),
                        ("Time Spent", "/timeSpent"),
                        ("Started", "/started"),
                        ("Comment", "/comment"),
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

        WorklogCommands::Edit {
            issue_key,
            worklog_id,
            time,
            started,
            body,
        } => {
            let request_body = build_edit_body(
                time.as_deref(),
                started.as_deref(),
                body.as_deref(),
            );

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("id".to_string(), worklog_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/worklog/{id}",
                &path_params,
                &[],
            );

            let result = http::execute_request(
                client,
                Method::PUT,
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
                        println!("Updated worklog {}", worklog_id);
                    }
                }
            }
            Ok(())
        }

        WorklogCommands::Delete {
            issue_key,
            worklog_id,
            yes,
        } => {
            if !yes
                && !super::confirm_delete_prompt(&format!(
                    "Delete worklog {} on {}? (y/N): ",
                    worklog_id, issue_key
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("id".to_string(), worklog_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/worklog/{id}",
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
                    println!("{}", json!({"id": worklog_id, "status": "deleted"}));
                }
                _ => println!("Deleted worklog {}", worklog_id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worklog_create_body() {
        let body = build_create_body("2h", Some("2024-01-15T09:00:00.000+0000"), None);
        assert_eq!(body["timeSpent"], "2h");
        assert_eq!(body["started"], "2024-01-15T09:00:00.000+0000");
        assert!(body.get("comment").is_none());
    }

    #[test]
    fn test_worklog_edit_body() {
        let body = build_edit_body(Some("3h"), None, Some("Updated notes"));
        assert_eq!(body["timeSpent"], "3h");
        assert!(body.get("started").is_none());
        assert!(body.get("comment").is_some());
        assert_eq!(body["comment"]["type"], "doc");
    }

    #[test]
    fn test_worklog_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-123".to_string());
        path_params.insert("id".to_string(), "54321".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/worklog/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-123/worklog/54321"
        );
    }

    #[test]
    fn test_worklog_remaining_query_params() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-456".to_string());
        let query_params = vec![
            ("adjustEstimate".to_string(), "new".to_string()),
            ("newEstimate".to_string(), "4h".to_string()),
        ];
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/worklog",
            &path_params,
            &query_params,
        );
        assert!(url.contains("/rest/api/3/issue/TEAM-456/worklog"));
        assert!(url.contains("adjustEstimate=new"));
        assert!(url.contains("newEstimate=4h"));
    }

    #[test]
    fn test_worklog_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-789".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/worklog",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-789/worklog"
        );
    }

    #[test]
    fn test_worklog_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-100".to_string());
        path_params.insert("id".to_string(), "99999".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/worklog/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-100/worklog/99999"
        );
    }
}
