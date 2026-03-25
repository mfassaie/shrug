//! Jira project component sub-entity: LCRUD operations.

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

/// Component subcommands.
#[derive(Subcommand)]
pub enum ComponentCommands {
    /// List components for a project
    List {
        /// Project key (e.g., TEAM)
        project_key: String,
    },
    /// Create a new component
    Create {
        /// Component name
        #[arg(long)]
        name: String,
        /// Project key (e.g., TEAM)
        #[arg(long)]
        project: String,
        /// Component description
        #[arg(long)]
        description: Option<String>,
        /// Lead account ID or @me
        #[arg(long)]
        lead: Option<String>,
        /// Assignee type (e.g., PROJECT_DEFAULT, COMPONENT_LEAD)
        #[arg(long)]
        assignee_type: Option<String>,
    },
    /// View a component
    View {
        /// Component ID
        id: String,
    },
    /// Edit a component
    Edit {
        /// Component ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New lead (accountId or @me)
        #[arg(long)]
        lead: Option<String>,
        /// New assignee type
        #[arg(long)]
        assignee_type: Option<String>,
    },
    /// Delete a component
    Delete {
        /// Component ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for component creation.
pub fn build_create_body(
    name: &str,
    project: &str,
    description: Option<&str>,
    lead: Option<&str>,
    assignee_type: Option<&str>,
) -> Value {
    let mut body = json!({
        "name": name,
        "project": project,
    });

    if let Some(d) = description {
        body["description"] = json!(d);
    }
    if let Some(l) = lead {
        body["leadAccountId"] = json!(l);
    }
    if let Some(at) = assignee_type {
        body["assigneeType"] = json!(at);
    }

    body
}

/// Build JSON request body for component edit. Only includes provided fields.
pub fn build_edit_body(
    name: Option<&str>,
    description: Option<&str>,
    lead: Option<&str>,
    assignee_type: Option<&str>,
) -> Value {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(d) = description {
        body["description"] = json!(d);
    }
    if let Some(l) = lead {
        body["leadAccountId"] = json!(l);
    }
    if let Some(at) = assignee_type {
        body["assigneeType"] = json!(at);
    }

    body
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a component command.
pub fn execute(
    cmd: &ComponentCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        ComponentCommands::List { project_key } => {
            let mut path_params = HashMap::new();
            path_params.insert("projectIdOrKey".to_string(), project_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/project/{projectIdOrKey}/component",
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

        ComponentCommands::Create {
            name,
            project,
            description,
            lead,
            assignee_type,
        } => {
            let resolved_lead = crate::jira::issue::resolve_at_me(
                lead.as_deref(),
                client,
                credential,
                base_url,
            )?;
            let request_body = build_create_body(
                name,
                project,
                description.as_deref(),
                resolved_lead.as_deref(),
                assignee_type.as_deref(),
            );

            let url = format!("{}/rest/api/3/component", base_url);
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
                            println!("Created component {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        ComponentCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/component/{id}",
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

        ComponentCommands::Edit {
            id,
            name,
            description,
            lead,
            assignee_type,
        } => {
            let resolved_lead = crate::jira::issue::resolve_at_me(
                lead.as_deref(),
                client,
                credential,
                base_url,
            )?;
            let request_body = build_edit_body(
                name.as_deref(),
                description.as_deref(),
                resolved_lead.as_deref(),
                assignee_type.as_deref(),
            );

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/component/{id}",
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
                _ => println!("Updated component {}", id),
            }
            Ok(())
        }

        ComponentCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete component {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/component/{id}",
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
                _ => println!("Deleted component {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_create_body() {
        let body = build_create_body(
            "Backend",
            "TEAM",
            Some("Backend services"),
            Some("abc123"),
            Some("PROJECT_DEFAULT"),
        );
        assert_eq!(body["name"], "Backend");
        assert_eq!(body["project"], "TEAM");
        assert_eq!(body["description"], "Backend services");
        assert_eq!(body["leadAccountId"], "abc123");
        assert_eq!(body["assigneeType"], "PROJECT_DEFAULT");
    }

    #[test]
    fn test_component_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("projectIdOrKey".to_string(), "TEAM".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/project/{projectIdOrKey}/component",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/project/TEAM/component"
        );
    }

    #[test]
    fn test_component_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "10042".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/component/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/component/10042"
        );
    }
}
