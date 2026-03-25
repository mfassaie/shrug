//! Jira Software epic agile entity: view, edit, list issues.

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

/// Epic agile entity subcommands.
#[derive(Subcommand)]
pub enum EpicCommands {
    /// View an epic
    View {
        /// Epic ID or key (e.g. TEAM-50)
        key: String,
    },
    /// Edit an epic (partial update via POST)
    Edit {
        /// Epic ID or key (e.g. TEAM-50)
        key: String,
        /// Mark epic as done
        #[arg(long)]
        done: bool,
        /// Mark epic as not done
        #[arg(long, conflicts_with = "done")]
        no_done: bool,
        /// Epic colour label key (e.g. ghx-label-3)
        #[arg(long)]
        color: Option<String>,
        /// New epic name
        #[arg(long)]
        name: Option<String>,
    },
    /// List issues in an epic
    List {
        /// Epic ID or key (e.g. TEAM-50)
        key: String,
        /// Filter to a specific board
        #[arg(long)]
        board: Option<u64>,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for epic edit. Only includes provided fields.
pub fn build_edit_body(
    done: bool,
    no_done: bool,
    color: Option<&str>,
    name: Option<&str>,
) -> Value {
    let mut body = json!({});

    if done {
        body["done"] = json!(true);
    } else if no_done {
        body["done"] = json!(false);
    }
    if let Some(c) = color {
        body["color"] = json!({"key": c});
    }
    if let Some(n) = name {
        body["name"] = json!(n);
    }

    body
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute an epic command.
pub fn execute(
    cmd: &EpicCommands,
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
        EpicCommands::View { key } => {
            let mut path_params = HashMap::new();
            path_params.insert("epicIdOrKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/rest/agile/1.0/epic/{epicIdOrKey}",
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

        EpicCommands::Edit {
            key,
            done,
            no_done,
            color: color_opt,
            name,
        } => {
            let request_body = build_edit_body(
                *done,
                *no_done,
                color_opt.as_deref(),
                name.as_deref(),
            );

            let mut path_params = HashMap::new();
            path_params.insert("epicIdOrKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/rest/agile/1.0/epic/{epicIdOrKey}",
                &path_params,
                &[],
            );

            // NOTE: The Jira agile API uses POST for epic partial updates, not PUT.
            http::execute_request(
                client,
                Method::POST,
                &url,
                Some(credential),
                Some(&request_body),
                &[],
            )?;

            match output_format {
                OutputFormat::Json => {
                    println!("{}", json!({"key": key, "status": "updated"}));
                }
                _ => println!("Updated epic {}", key),
            }
            Ok(())
        }

        EpicCommands::List { key, board } => {
            let mut query_params: Vec<(String, String)> = Vec::new();
            if let Some(b) = board {
                query_params.push(("boardId".to_string(), b.to_string()));
            }

            let mut path_params = HashMap::new();
            path_params.insert("epicIdOrKey".to_string(), key.clone());
            let url_base = http::build_url(
                &base_url, "/rest/agile/1.0/epic/{epicIdOrKey}/issue",
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_epic_edit_done() {
        let body = build_edit_body(true, false, None, None);
        assert_eq!(body["done"], true);
    }

    #[test]
    fn test_epic_edit_color() {
        let body = build_edit_body(false, false, Some("ghx-label-3"), None);
        assert_eq!(body["color"]["key"], "ghx-label-3");
    }

    #[test]
    fn test_epic_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("epicIdOrKey".to_string(), "TEAM-50".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/epic/{epicIdOrKey}/issue",
            &path_params,
            &[],
        );
        assert!(url.contains("/rest/agile/1.0/epic/TEAM-50/issue"));
    }

    #[test]
    fn test_epic_done_and_no_done_conflict() {
        let result = crate::cli::Cli::try_parse_from([
            "shrug",
            "jira-software",
            "epic",
            "edit",
            "TEAM-50",
            "--done",
            "--no-done",
        ]);
        assert!(result.is_err(), "--done and --no-done should conflict");
    }

    #[test]
    fn test_epic_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("epicIdOrKey".to_string(), "TEAM-99".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/agile/1.0/epic/{epicIdOrKey}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/agile/1.0/epic/TEAM-99"
        );
    }
}
