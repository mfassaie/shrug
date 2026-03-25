//! Jira dashboard entity: LCRUD operations.

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

/// Dashboard entity subcommands.
#[derive(Subcommand)]
pub enum DashboardCommands {
    /// List dashboards
    List {
        /// Filter by name
        #[arg(long)]
        name: Option<String>,
        /// Filter by owner (accountId)
        #[arg(long)]
        owner: Option<String>,
        /// Order results
        #[arg(long)]
        order_by: Option<String>,
    },
    /// Create a new dashboard
    Create {
        /// Dashboard name
        #[arg(long)]
        name: String,
        /// Dashboard description
        #[arg(long)]
        description: Option<String>,
        /// Share permission type (e.g., global, project, group)
        #[arg(long)]
        share: Option<String>,
    },
    /// View a dashboard
    View {
        /// Dashboard ID
        id: String,
    },
    /// Edit a dashboard
    Edit {
        /// Dashboard ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a dashboard
    Delete {
        /// Dashboard ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for dashboard creation.
pub fn build_create_body(
    name: &str,
    description: Option<&str>,
    share: Option<&str>,
) -> Value {
    let mut body = json!({
        "name": name,
    });

    if let Some(d) = description {
        body["description"] = json!(d);
    }
    if let Some(s) = share {
        body["sharePermissions"] = json!([{"type": s}]);
    }

    body
}

/// Build JSON request body for dashboard edit. Only includes provided fields.
pub fn build_edit_body(
    name: Option<&str>,
    description: Option<&str>,
) -> Value {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(d) = description {
        body["description"] = json!(d);
    }

    body
}

/// Build query parameters for dashboard list.
pub fn build_list_query_params(
    name: Option<&str>,
    owner: Option<&str>,
    order_by: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(n) = name {
        params.push(("filter".to_string(), n.to_string()));
    }
    if let Some(o) = owner {
        params.push(("accountId".to_string(), o.to_string()));
    }
    if let Some(ob) = order_by {
        params.push(("orderBy".to_string(), ob.to_string()));
    }
    params
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a dashboard command.
pub fn execute(
    cmd: &DashboardCommands,
    credential: &ResolvedCredential,
    client: &Client,
    output_format: &OutputFormat,
    color: &ColorChoice,
    limit: Option<u32>,
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
        DashboardCommands::List {
            name,
            owner,
            order_by,
        } => {
            let mut query_params = build_list_query_params(
                name.as_deref(),
                owner.as_deref(),
                order_by.as_deref(),
            );
            if let Some(lim) = limit {
                query_params.push(("maxResults".to_string(), lim.to_string()));
            }

            let url = http::build_url(
                &base_url,
                "/rest/api/3/dashboard/search",
                &HashMap::new(),
                &query_params,
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

        DashboardCommands::Create {
            name,
            description,
            share,
        } => {
            let request_body =
                build_create_body(name, description.as_deref(), share.as_deref());

            let url = format!("{}/rest/api/3/dashboard", base_url);
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
                            println!("Created dashboard {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        DashboardCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/dashboard/{id}",
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

        DashboardCommands::Edit {
            id,
            name,
            description,
        } => {
            let request_body =
                build_edit_body(name.as_deref(), description.as_deref());

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/dashboard/{id}",
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
                _ => println!("Updated dashboard {}", id),
            }
            Ok(())
        }

        DashboardCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete dashboard {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/dashboard/{id}",
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
                _ => println!("Deleted dashboard {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_create_body() {
        let body = build_create_body("Sprint Board", Some("Team dashboard"), Some("global"));
        assert_eq!(body["name"], "Sprint Board");
        assert_eq!(body["description"], "Team dashboard");
        assert_eq!(body["sharePermissions"][0]["type"], "global");
    }

    #[test]
    fn test_dashboard_list_url() {
        let query_params =
            build_list_query_params(Some("sprint"), None, Some("name"));
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/dashboard/search",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/rest/api/3/dashboard/search"));
        assert!(url.contains("filter=sprint"));
        assert!(url.contains("orderBy=name"));
    }
}
