//! Jira project version sub-entity: LCRUD operations.

use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::auth::credentials::ResolvedCredential;
use crate::cli::OutputFormat;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Version subcommands.
#[derive(Subcommand)]
pub enum VersionCommands {
    /// List versions for a project
    List {
        /// Project key (e.g., TEAM)
        project_key: String,
        /// Order results (e.g., name, -name, sequence)
        #[arg(long)]
        order_by: Option<String>,
        /// Filter by status (e.g., released, unreleased)
        #[arg(long)]
        status: Option<String>,
    },
    /// Create a new version
    Create {
        /// Version name (e.g., v2.1)
        #[arg(long)]
        name: String,
        /// Project key (e.g., TEAM)
        #[arg(long)]
        project: String,
        /// Version description
        #[arg(long)]
        description: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,
        /// Release date (YYYY-MM-DD)
        #[arg(long)]
        release_date: Option<String>,
        /// Mark as released
        #[arg(long)]
        released: bool,
        /// Mark as archived
        #[arg(long)]
        archived: bool,
    },
    /// View a version
    View {
        /// Version ID
        id: String,
    },
    /// Edit a version
    Edit {
        /// Version ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,
        /// New release date (YYYY-MM-DD)
        #[arg(long)]
        release_date: Option<String>,
        /// Set released state
        #[arg(long)]
        released: bool,
        /// Set archived state
        #[arg(long)]
        archived: bool,
    },
    /// Delete a version
    Delete {
        /// Version ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for version creation.
///
/// Boolean flags (released, archived) are only included when true.
pub fn build_create_body(
    name: &str,
    project: &str,
    description: Option<&str>,
    start_date: Option<&str>,
    release_date: Option<&str>,
    released: bool,
    archived: bool,
) -> Value {
    let mut body = json!({
        "name": name,
        "project": project,
    });

    if let Some(d) = description {
        body["description"] = json!(d);
    }
    if let Some(sd) = start_date {
        body["startDate"] = json!(sd);
    }
    if let Some(rd) = release_date {
        body["releaseDate"] = json!(rd);
    }
    if released {
        body["released"] = json!(true);
    }
    if archived {
        body["archived"] = json!(true);
    }

    body
}

/// Build JSON request body for version edit. Only includes provided fields.
///
/// Boolean flags are only included when true (edit sets them explicitly).
#[allow(clippy::too_many_arguments)]
pub fn build_edit_body(
    name: Option<&str>,
    description: Option<&str>,
    start_date: Option<&str>,
    release_date: Option<&str>,
    released: bool,
    archived: bool,
) -> Value {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(d) = description {
        body["description"] = json!(d);
    }
    if let Some(sd) = start_date {
        body["startDate"] = json!(sd);
    }
    if let Some(rd) = release_date {
        body["releaseDate"] = json!(rd);
    }
    if released {
        body["released"] = json!(true);
    }
    if archived {
        body["archived"] = json!(true);
    }

    body
}

/// Build query parameters for version list.
pub fn build_list_query_params(
    order_by: Option<&str>,
    status: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(o) = order_by {
        params.push(("orderBy".to_string(), o.to_string()));
    }
    if let Some(s) = status {
        params.push(("status".to_string(), s.to_string()));
    }
    params
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a version command.
pub fn execute(
    cmd: &VersionCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
    dry_run: bool,
) -> Result<(), ShrugError> {
    match cmd {
        VersionCommands::List {
            project_key,
            order_by,
            status,
        } => {
            let query_params =
                build_list_query_params(order_by.as_deref(), status.as_deref());
            let mut path_params = HashMap::new();
            path_params.insert("projectIdOrKey".to_string(), project_key.clone());
            let url_base = http::build_url(
                base_url,
                "/rest/api/3/project/{projectIdOrKey}/version",
                &path_params,
                &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url_base, None);
                return Ok(());
            }

            let results = http::execute_paginated_get(
                client, &url_base, credential, &query_params, &[], None, 50, false,
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

        VersionCommands::Create {
            name,
            project,
            description,
            start_date,
            release_date,
            released,
            archived,
        } => {
            let request_body = build_create_body(
                name,
                project,
                description.as_deref(),
                start_date.as_deref(),
                release_date.as_deref(),
                *released,
                *archived,
            );

            let url = format!("{}/rest/api/3/version", base_url);

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
                        if let Some(id) = json_val.get("id").and_then(|v| v.as_str()) {
                            println!("Created version {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        VersionCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/version/{id}",
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

        VersionCommands::Edit {
            id,
            name,
            description,
            start_date,
            release_date,
            released,
            archived,
        } => {
            let request_body = build_edit_body(
                name.as_deref(),
                description.as_deref(),
                start_date.as_deref(),
                release_date.as_deref(),
                *released,
                *archived,
            );

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/version/{id}",
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
                    println!("{}", json!({"id": id, "status": "updated"}));
                }
                _ => println!("Updated version {}", id),
            }
            Ok(())
        }

        VersionCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete version {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/version/{id}",
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
                _ => println!("Deleted version {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_create_body() {
        let body = build_create_body(
            "v2.1",
            "TEAM",
            Some("Sprint release"),
            Some("2025-01-01"),
            Some("2025-03-01"),
            true,
            false,
        );
        assert_eq!(body["name"], "v2.1");
        assert_eq!(body["project"], "TEAM");
        assert_eq!(body["description"], "Sprint release");
        assert_eq!(body["startDate"], "2025-01-01");
        assert_eq!(body["releaseDate"], "2025-03-01");
        assert_eq!(body["released"], true);
        assert!(body.get("archived").is_none());
    }

    #[test]
    fn test_version_list_url() {
        let query_params = build_list_query_params(Some("name"), Some("unreleased"));
        let mut path_params = HashMap::new();
        path_params.insert("projectIdOrKey".to_string(), "TEAM".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/project/{projectIdOrKey}/version",
            &path_params,
            &query_params,
        );
        assert!(url.contains("/rest/api/3/project/TEAM/version"));
        assert!(url.contains("orderBy=name"));
        assert!(url.contains("status=unreleased"));
    }

    #[test]
    fn test_version_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "10001".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/version/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/version/10001"
        );
    }

    #[test]
    fn test_version_edit_body() {
        let body = build_edit_body(
            Some("v3.0"),
            None,
            None,
            Some("2025-06-01"),
            true,
            false,
        );
        assert_eq!(body["name"], "v3.0");
        assert!(body.get("description").is_none());
        assert!(body.get("startDate").is_none());
        assert_eq!(body["releaseDate"], "2025-06-01");
        assert_eq!(body["released"], true);
        assert!(body.get("archived").is_none());
    }

    #[test]
    fn test_version_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "20001".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/version/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/version/20001"
        );
    }
}
