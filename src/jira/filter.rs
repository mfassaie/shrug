//! Jira filter entity: LCRUD operations for saved JQL filters.

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

/// Filter entity subcommands.
#[derive(Subcommand)]
pub enum FilterCommands {
    /// List filters
    List {
        /// Filter by name
        #[arg(long)]
        name: Option<String>,
        /// Filter by owner (accountId)
        #[arg(long)]
        owner: Option<String>,
        /// Show only favourite filters
        #[arg(long)]
        favourites: bool,
        /// Order results
        #[arg(long)]
        order_by: Option<String>,
    },
    /// Create a new filter
    Create {
        /// Filter name
        #[arg(long)]
        name: String,
        /// JQL query
        #[arg(long)]
        jql: String,
        /// Filter description
        #[arg(long)]
        description: Option<String>,
        /// Mark as favourite
        #[arg(long)]
        favourite: bool,
    },
    /// View a filter
    View {
        /// Filter ID
        id: String,
    },
    /// Edit a filter
    Edit {
        /// Filter ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New JQL query
        #[arg(long)]
        jql: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a filter
    Delete {
        /// Filter ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for filter creation.
pub fn build_create_body(
    name: &str,
    jql: &str,
    description: Option<&str>,
    favourite: bool,
) -> Value {
    let mut body = json!({
        "name": name,
        "jql": jql,
    });

    if let Some(d) = description {
        body["description"] = json!(d);
    }
    if favourite {
        body["favourite"] = json!(true);
    }

    body
}

/// Build JSON request body for filter edit. Only includes provided fields.
pub fn build_edit_body(
    name: Option<&str>,
    jql: Option<&str>,
    description: Option<&str>,
) -> Value {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(j) = jql {
        body["jql"] = json!(j);
    }
    if let Some(d) = description {
        body["description"] = json!(d);
    }

    body
}

/// Build query parameters for filter list.
pub fn build_list_query_params(
    name: Option<&str>,
    owner: Option<&str>,
    favourites: bool,
    order_by: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(n) = name {
        params.push(("filterName".to_string(), n.to_string()));
    }
    if let Some(o) = owner {
        params.push(("accountId".to_string(), o.to_string()));
    }
    if favourites {
        params.push(("favourite".to_string(), "true".to_string()));
    }
    if let Some(ob) = order_by {
        params.push(("orderBy".to_string(), ob.to_string()));
    }
    params
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a filter command.
pub fn execute(
    cmd: &FilterCommands,
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
        FilterCommands::List {
            name,
            owner,
            favourites,
            order_by,
        } => {
            let query_params = build_list_query_params(
                name.as_deref(),
                owner.as_deref(),
                *favourites,
                order_by.as_deref(),
            );
            let url_base = http::build_url(
                &base_url,
                "/rest/api/3/filter/search",
                &HashMap::new(),
                &[],
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

        FilterCommands::Create {
            name,
            jql,
            description,
            favourite,
        } => {
            let request_body = build_create_body(name, jql, description.as_deref(), *favourite);

            let url = format!("{}/rest/api/3/filter", base_url);
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
                            println!("Created filter {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        FilterCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/filter/{id}",
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

        FilterCommands::Edit {
            id,
            name,
            jql,
            description,
        } => {
            let request_body = build_edit_body(
                name.as_deref(),
                jql.as_deref(),
                description.as_deref(),
            );

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/filter/{id}",
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
                _ => println!("Updated filter {}", id),
            }
            Ok(())
        }

        FilterCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete filter {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/filter/{id}",
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
                _ => println!("Deleted filter {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_create_body() {
        let body = build_create_body(
            "My Bugs",
            "project = TEAM AND type = Bug",
            Some("All bugs in TEAM"),
            true,
        );
        assert_eq!(body["name"], "My Bugs");
        assert_eq!(body["jql"], "project = TEAM AND type = Bug");
        assert_eq!(body["description"], "All bugs in TEAM");
        assert_eq!(body["favourite"], true);
    }

    #[test]
    fn test_filter_list_url() {
        let query_params =
            build_list_query_params(Some("bugs"), None, true, Some("name"));
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/filter/search",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/rest/api/3/filter/search"));
        assert!(url.contains("filterName=bugs"));
        assert!(url.contains("favourite=true"));
        assert!(url.contains("orderBy=name"));
    }

    #[test]
    fn test_filter_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "10100".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/filter/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/filter/10100"
        );
    }

    #[test]
    fn test_filter_edit_body() {
        let body = build_edit_body(
            Some("Updated Filter"),
            Some("project = NEW"),
            None,
        );
        assert_eq!(body["name"], "Updated Filter");
        assert_eq!(body["jql"], "project = NEW");
        assert!(body.get("description").is_none());
    }

    #[test]
    fn test_filter_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "20100".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/filter/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/filter/20100"
        );
    }
}
