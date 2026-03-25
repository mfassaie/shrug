//! Confluence space entity: LCRUD operations (mixed v1/v2 API).

pub mod property;

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
use crate::jira::issue::read_json_file;

/// Space entity subcommands.
#[derive(Subcommand)]
pub enum SpaceCommands {
    /// List spaces
    List {
        /// Space type (global, personal)
        #[arg(long = "type")]
        space_type: Option<String>,
        /// Space status (current, archived)
        #[arg(long)]
        status: Option<String>,
        /// Filter by space key
        #[arg(long)]
        query: Option<String>,
    },
    /// Create a new space
    Create {
        /// Space key (e.g., DOCS)
        #[arg(short = 'k', long)]
        key: String,
        /// Space name
        #[arg(long)]
        name: String,
        /// Space description
        #[arg(long)]
        description: Option<String>,
        /// Space type (global, personal)
        #[arg(long = "type")]
        space_type: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// View a space by ID
    View {
        /// Space ID (numeric)
        id: String,
    },
    /// Edit a space by key (v1 API)
    Edit {
        /// Space key (e.g., DOCS)
        key: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// Delete a space by key (v1 API)
    Delete {
        /// Space key (e.g., DOCS)
        key: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Space property operations
    Property {
        #[command(subcommand)]
        command: property::SpacePropertyCommands,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for space creation (v2).
pub fn build_create_body(
    key: &str,
    name: &str,
    description: Option<&str>,
    space_type: Option<&str>,
) -> Value {
    let mut body = json!({
        "key": key,
        "name": name,
    });

    if let Some(d) = description {
        body["description"] = json!({
            "plain": {
                "value": d,
                "representation": "plain",
            }
        });
    }

    body["type"] = json!(space_type.unwrap_or("global"));

    body
}

/// Build JSON request body for space edit (v1).
pub fn build_edit_body(name: Option<&str>, description: Option<&str>) -> Value {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(d) = description {
        body["description"] = json!({
            "plain": {
                "value": d,
                "representation": "plain",
            }
        });
    }

    body
}

/// Build query parameters for space list (v2).
pub fn build_list_query_params(
    space_type: Option<&str>,
    status: Option<&str>,
    query: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(t) = space_type {
        params.push(("type".to_string(), t.to_string()));
    }
    if let Some(s) = status {
        params.push(("status".to_string(), s.to_string()));
    }
    if let Some(q) = query {
        params.push(("keys".to_string(), q.to_string()));
    }
    params
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a space command.
pub fn execute(
    cmd: &SpaceCommands,
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
        SpaceCommands::List {
            space_type,
            status,
            query,
        } => {
            let query_params = build_list_query_params(
                space_type.as_deref(),
                status.as_deref(),
                query.as_deref(),
            );
            let url_base = http::build_url(
                &base_url, "/wiki/api/v2/spaces", &HashMap::new(), &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url_base, None);
                return Ok(());
            }

            let results = http::execute_paginated_get(
                client, &url_base, credential, &query_params, &[], limit, 25, true,
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

        SpaceCommands::Create {
            key,
            name,
            description,
            space_type,
            from_json,
        } => {
            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                read_json_file(path)?
            } else {
                build_create_body(key, name, description.as_deref(), space_type.as_deref())
            };

            let url = format!("{}/wiki/api/v2/spaces", base_url);

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
                            println!("Created space {}", id);
                        } else if let Some(k) = json_val.get("key").and_then(|v| v.as_str()) {
                            println!("Created space {}", k);
                        }
                    }
                }
            }
            Ok(())
        }

        SpaceCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/spaces/{id}",
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

        SpaceCommands::Edit {
            key,
            name,
            description,
            from_json,
        } => {
            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                read_json_file(path)?
            } else {
                build_edit_body(name.as_deref(), description.as_deref())
            };

            // Space edit uses v1 API with space key
            let mut path_params = HashMap::new();
            path_params.insert("spaceKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/rest/api/space/{spaceKey}",
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
                    println!("{}", json!({"key": key, "status": "updated"}));
                }
                _ => println!("Updated space {}", key),
            }
            Ok(())
        }

        SpaceCommands::Delete { key, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete space {}? (y/N): ",
                    key
                ))?
            {
                return Ok(());
            }

            // Space delete uses v1 API with space key
            let mut path_params = HashMap::new();
            path_params.insert("spaceKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/rest/api/space/{spaceKey}",
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
                    println!("{}", json!({"key": key, "status": "deleted"}));
                }
                _ => println!("Deleted space {}", key),
            }
            Ok(())
        }

        SpaceCommands::Property { command } => {
            property::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_create_body() {
        let body = build_create_body("DOCS", "Documentation", Some("Team docs"), None);
        assert_eq!(body["key"], "DOCS");
        assert_eq!(body["name"], "Documentation");
        assert_eq!(body["description"]["plain"]["value"], "Team docs");
        assert_eq!(body["description"]["plain"]["representation"], "plain");
        assert_eq!(body["type"], "global");
    }

    #[test]
    fn test_space_edit_url() {
        let mut path_params = HashMap::new();
        path_params.insert("spaceKey".to_string(), "DOCS".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/rest/api/space/{spaceKey}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/rest/api/space/DOCS"
        );
    }

    #[test]
    fn test_space_list_url() {
        let query_params = build_list_query_params(Some("global"), Some("current"), None);
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/spaces",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/wiki/api/v2/spaces"));
        assert!(url.contains("type=global"));
        assert!(url.contains("status=current"));
    }

    #[test]
    fn test_space_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "98765".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/spaces/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/spaces/98765"
        );
    }

    #[test]
    fn test_space_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("spaceKey".to_string(), "OLD".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/rest/api/space/{spaceKey}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/rest/api/space/OLD"
        );
    }

    #[test]
    fn test_space_create_with_from_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("space.json");
        std::fs::write(&path, r#"{"key":"TEST","name":"Test Space","type":"global"}"#).unwrap();
        let value = crate::jira::issue::read_json_file(path.to_str().unwrap()).unwrap();
        assert_eq!(value["key"], "TEST");
        assert_eq!(value["name"], "Test Space");
    }
}
