//! Jira Software board entity: list, create, view, delete operations.

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

/// Board entity subcommands.
#[derive(Subcommand)]
pub enum BoardCommands {
    /// List boards
    List {
        /// Filter by project key or ID
        #[arg(long)]
        project: Option<String>,
        /// Board type (scrum, kanban)
        #[arg(long = "type")]
        board_type: Option<String>,
        /// Filter by board name
        #[arg(long)]
        name: Option<String>,
    },
    /// Create a new board
    Create {
        /// Board name
        #[arg(long)]
        name: String,
        /// Board type (scrum, kanban)
        #[arg(long = "type")]
        board_type: String,
        /// Saved filter ID to back this board
        #[arg(long)]
        filter_id: u64,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// View a board
    View {
        /// Board ID
        id: String,
    },
    /// Delete a board
    Delete {
        /// Board ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// View board configuration
    Config {
        /// Board ID
        id: String,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for board creation.
pub fn build_create_body(name: &str, board_type: &str, filter_id: u64) -> Value {
    json!({
        "name": name,
        "type": board_type,
        "filterId": filter_id,
    })
}

/// Build query parameters for board list.
pub fn build_list_query_params(
    project: Option<&str>,
    board_type: Option<&str>,
    name: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(p) = project {
        params.push(("projectKeyOrId".to_string(), p.to_string()));
    }
    if let Some(t) = board_type {
        params.push(("type".to_string(), t.to_string()));
    }
    if let Some(n) = name {
        params.push(("name".to_string(), n.to_string()));
    }
    params
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a board command.
pub fn execute(
    cmd: &BoardCommands,
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
        BoardCommands::List {
            project,
            board_type,
            name,
        } => {
            let query_params = build_list_query_params(
                project.as_deref(),
                board_type.as_deref(),
                name.as_deref(),
            );
            let url_base = http::build_url(
                &base_url, "/rest/agile/1.0/board", &HashMap::new(), &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url_base, None);
                return Ok(());
            }

            let results = http::execute_paginated_get(
                client, &url_base, credential, &query_params, &[], limit, 50, false,
            )?;
            if !results.is_empty() {
                let json_val = if matches!(output_format, OutputFormat::Json) {
                    serde_json::Value::Array(results)
                } else {
                    output::project_array(&results, &[
                        ("ID", "/id"),
                        ("Name", "/name"),
                        ("Type", "/type"),
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

        BoardCommands::Create {
            name,
            board_type,
            filter_id,
            from_json,
        } => {
            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                crate::jira::issue::read_json_file(path)?
            } else {
                build_create_body(name, board_type, *filter_id)
            };

            let url = format!("{}/rest/agile/1.0/board", base_url);

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
                            println!("Created board {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        BoardCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("boardId".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/agile/1.0/board/{boardId}",
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
                        ("Name", "/name"),
                        ("Type", "/type"),
                        ("Location", "/location/name"),
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

        BoardCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete board {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("boardId".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/agile/1.0/board/{boardId}",
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
                _ => println!("Deleted board {}", id),
            }
            Ok(())
        }

        BoardCommands::Config { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("boardId".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/agile/1.0/board/{boardId}/configuration",
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
                        ("Name", "/name"),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_create_body() {
        let body = build_create_body("My Board", "scrum", 10200);
        assert_eq!(body["name"], "My Board");
        assert_eq!(body["type"], "scrum");
        assert_eq!(body["filterId"], 10200);
    }

    #[test]
    fn test_board_list_url() {
        let query_params =
            build_list_query_params(Some("TEAM"), Some("scrum"), Some("Sprint"));
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/board",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/rest/agile/1.0/board"));
        assert!(url.contains("projectKeyOrId=TEAM"));
        assert!(url.contains("type=scrum"));
        assert!(url.contains("name=Sprint"));
    }

    #[test]
    fn test_board_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("boardId".to_string(), "42".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/board/{boardId}",
            &path_params,
            &[],
        );
        assert!(url.contains("/rest/agile/1.0/board/42"));
    }

    #[test]
    fn test_board_config_url() {
        let mut path_params = HashMap::new();
        path_params.insert("boardId".to_string(), "42".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/board/{boardId}/configuration",
            &path_params,
            &[],
        );
        assert!(url.contains("/rest/agile/1.0/board/42/configuration"));
    }

    #[test]
    fn test_board_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("boardId".to_string(), "99".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/board/{boardId}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/agile/1.0/board/99"
        );
    }

    #[test]
    fn test_board_create_with_from_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("board.json");
        std::fs::write(&path, r#"{"name":"Custom","type":"kanban","filterId":999}"#).unwrap();
        let value = crate::jira::issue::read_json_file(path.to_str().unwrap()).unwrap();
        assert_eq!(value["name"], "Custom");
        assert_eq!(value["type"], "kanban");
        assert_eq!(value["filterId"], 999);
    }
}
