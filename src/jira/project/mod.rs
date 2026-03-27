//! Jira project entity: LCRUD operations plus component and version sub-entities.

pub mod component;
pub mod version;

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

/// Project entity subcommands.
#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List projects
    List {
        /// Search query
        #[arg(long)]
        query: Option<String>,
        /// Project type filter (e.g., software, business)
        #[arg(long = "type")]
        project_type: Option<String>,
        /// Order results (e.g., name, -name)
        #[arg(long)]
        order_by: Option<String>,
    },
    /// Create a new project
    Create {
        /// Project key (e.g., TEAM)
        #[arg(short = 'k', long)]
        key: String,
        /// Project name
        #[arg(long)]
        name: String,
        /// Project type (e.g., software, business)
        #[arg(long = "type")]
        project_type: String,
        /// Lead account ID or @me
        #[arg(long)]
        lead: Option<String>,
        /// Project template key
        #[arg(long)]
        template: Option<String>,
        /// Project description
        #[arg(long)]
        description: Option<String>,
    },
    /// View a project
    View {
        /// Project key or ID
        key: String,
    },
    /// Edit a project
    Edit {
        /// Project key or ID
        key: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New lead (accountId or @me)
        #[arg(long)]
        lead: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New URL
        #[arg(long)]
        url: Option<String>,
    },
    /// Delete a project
    Delete {
        /// Project key or ID
        key: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Component operations on a project
    Component {
        #[command(subcommand)]
        command: component::ComponentCommands,
    },
    /// Version operations on a project
    Version {
        #[command(subcommand)]
        command: version::VersionCommands,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for project creation.
pub fn build_create_body(
    key: &str,
    name: &str,
    project_type: &str,
    lead: Option<&str>,
    template: Option<&str>,
    description: Option<&str>,
) -> Value {
    let mut body = json!({
        "key": key,
        "name": name,
        "projectTypeKey": project_type,
    });

    if let Some(l) = lead {
        body["leadAccountId"] = json!(l);
    }
    if let Some(t) = template {
        body["projectTemplateKey"] = json!(t);
    }
    if let Some(d) = description {
        body["description"] = json!(d);
    }

    body
}

/// Build JSON request body for project edit. Only includes provided fields.
pub fn build_edit_body(
    name: Option<&str>,
    lead: Option<&str>,
    description: Option<&str>,
    url: Option<&str>,
) -> Value {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(l) = lead {
        body["leadAccountId"] = json!(l);
    }
    if let Some(d) = description {
        body["description"] = json!(d);
    }
    if let Some(u) = url {
        body["url"] = json!(u);
    }

    body
}

/// Build query parameters for project list.
pub fn build_list_query_params(
    query: Option<&str>,
    project_type: Option<&str>,
    order_by: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(q) = query {
        params.push(("query".to_string(), q.to_string()));
    }
    if let Some(t) = project_type {
        params.push(("typeKey".to_string(), t.to_string()));
    }
    if let Some(o) = order_by {
        params.push(("orderBy".to_string(), o.to_string()));
    }
    params
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a project command.
pub fn execute(
    cmd: &ProjectCommands,
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
        ProjectCommands::List {
            query,
            project_type,
            order_by,
        } => {
            let query_params = build_list_query_params(
                query.as_deref(),
                project_type.as_deref(),
                order_by.as_deref(),
            );
            let url_base = http::build_url(
                &base_url,
                "/rest/api/3/project/search",
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
            if !results.is_empty() {
                let json_val = if matches!(output_format, OutputFormat::Json) {
                    serde_json::Value::Array(results)
                } else {
                    output::project_array(&results, &[
                        ("Key", "/key"),
                        ("Name", "/name"),
                        ("Type", "/projectTypeKey"),
                        ("Lead", "/lead"),
                    ])
                };
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

        ProjectCommands::Create {
            key,
            name,
            project_type,
            lead,
            template,
            description,
        } => {
            let resolved_lead =
                crate::jira::issue::resolve_at_me(lead.as_deref(), client, credential, &base_url)?;
            let request_body = build_create_body(
                key,
                name,
                project_type,
                resolved_lead.as_deref(),
                template.as_deref(),
                description.as_deref(),
            );

            let url = format!("{}/rest/api/3/project", base_url);

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
                        if let Some(key_val) = json_val.get("key").and_then(|v| v.as_str()) {
                            println!("Created {}", key_val);
                        }
                    }
                }
            }
            Ok(())
        }

        ProjectCommands::View { key } => {
            let mut path_params = HashMap::new();
            path_params.insert("projectIdOrKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/project/{projectIdOrKey}",
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
                        ("Key", "/key"),
                        ("Name", "/name"),
                        ("Type", "/projectTypeKey"),
                        ("Lead", "/lead"),
                        ("Description", "/description"),
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

        ProjectCommands::Edit {
            key,
            name,
            lead,
            description,
            url: project_url,
        } => {
            let resolved_lead =
                crate::jira::issue::resolve_at_me(lead.as_deref(), client, credential, &base_url)?;
            let request_body = build_edit_body(
                name.as_deref(),
                resolved_lead.as_deref(),
                description.as_deref(),
                project_url.as_deref(),
            );

            let mut path_params = HashMap::new();
            path_params.insert("projectIdOrKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/project/{projectIdOrKey}",
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
                    println!("{}", json!({"key": key, "status": "updated"}));
                }
                _ => println!("Updated {}", key),
            }
            Ok(())
        }

        ProjectCommands::Delete { key, yes } => {
            let mut path_params = HashMap::new();
            path_params.insert("projectIdOrKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/project/{projectIdOrKey}",
                &path_params,
                &[],
            );

            if dry_run {
                http::dry_run_request(&Method::DELETE, &url, None);
                return Ok(());
            }

            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete project {}? (y/N): ",
                    key
                ))?
            {
                return Ok(());
            }

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
                _ => println!("Deleted {}", key),
            }
            Ok(())
        }

        ProjectCommands::Component { command } => {
            component::execute(command, credential, client, &base_url, output_format, color_enabled, dry_run)
        }
        ProjectCommands::Version { command } => {
            version::execute(command, credential, client, &base_url, output_format, color_enabled, dry_run)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_create_body() {
        let body = build_create_body(
            "TEAM",
            "My Team Project",
            "software",
            Some("abc123"),
            Some("com.pyxis.greenhopper.jira:gh-simplified-agility-kanban"),
            Some("A test project"),
        );
        assert_eq!(body["key"], "TEAM");
        assert_eq!(body["name"], "My Team Project");
        assert_eq!(body["projectTypeKey"], "software");
        assert_eq!(body["leadAccountId"], "abc123");
        assert_eq!(
            body["projectTemplateKey"],
            "com.pyxis.greenhopper.jira:gh-simplified-agility-kanban"
        );
        assert_eq!(body["description"], "A test project");
    }

    #[test]
    fn test_project_edit_body() {
        let body = build_edit_body(Some("New Name"), None, Some("Updated desc"), None);
        assert_eq!(body["name"], "New Name");
        assert_eq!(body["description"], "Updated desc");
        assert!(body.get("leadAccountId").is_none());
        assert!(body.get("url").is_none());
    }

    #[test]
    fn test_project_list_url() {
        let query_params = build_list_query_params(Some("team"), Some("software"), None);
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/project/search",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/rest/api/3/project/search"));
        assert!(url.contains("query=team"));
        assert!(url.contains("typeKey=software"));
    }

    #[test]
    fn test_project_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("projectIdOrKey".to_string(), "TEAM".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/project/{projectIdOrKey}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/project/TEAM"
        );
    }

    #[test]
    fn test_project_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("projectIdOrKey".to_string(), "OLD".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/project/{projectIdOrKey}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/project/OLD"
        );
    }
}
