//! Jira issue link sub-entity: list, create, view, delete.
//!
//! Issue links use a different API path (/rest/api/3/issueLink) from other
//! sub-entities. Listing fetches the parent issue with ?fields=issuelinks.

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

/// Issue link subcommands.
#[derive(Subcommand)]
pub enum LinkCommands {
    /// List links on an issue
    List {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
    },
    /// Create a link between two issues
    Create {
        /// Outward issue key (the "from" side)
        #[arg(long)]
        from: String,
        /// Inward issue key (the "to" side)
        #[arg(long)]
        to: String,
        /// Link type name (e.g., blocks, duplicates, relates to)
        #[arg(long = "type")]
        link_type: String,
        /// Optional comment in markdown
        #[arg(short = 'b', long)]
        body: Option<String>,
    },
    /// View a specific issue link
    View {
        /// Issue link ID
        link_id: String,
    },
    /// Delete an issue link
    Delete {
        /// Issue link ID
        link_id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Build the issue link create body.
pub fn build_create_body(
    from: &str,
    to: &str,
    link_type: &str,
    body: Option<&str>,
) -> Value {
    let mut payload = json!({
        "type": { "name": link_type },
        "outwardIssue": { "key": from },
        "inwardIssue": { "key": to },
    });

    if let Some(b) = body {
        payload["comment"] = json!({
            "body": markdown_to_adf::markdown_to_adf(b),
        });
    }

    payload
}

/// Execute a link command.
pub fn execute(
    cmd: &LinkCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        LinkCommands::List { issue_key } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}",
                &path_params,
                &[("fields".to_string(), "issuelinks".to_string())],
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
                let links = json_val
                    .get("fields")
                    .and_then(|f| f.get("issuelinks"))
                    .cloned()
                    .unwrap_or(Value::Array(vec![]));
                let formatted = output::format_response(
                    &links.to_string(),
                    output_format,
                    is_terminal::is_terminal(std::io::stdout()),
                    color_enabled,
                    None,
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        LinkCommands::Create {
            from,
            to,
            link_type,
            body,
        } => {
            let request_body = build_create_body(from, to, link_type, body.as_deref());

            let url = http::build_url(
                base_url,
                "/rest/api/3/issueLink",
                &HashMap::new(),
                &[],
            );

            http::execute_request(
                client,
                Method::POST,
                &url,
                Some(credential),
                Some(&request_body),
                &[],
            )?;

            // POST /issueLink returns 201 with no body on success
            match output_format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        json!({"from": from, "to": to, "type": link_type, "status": "created"})
                    );
                }
                _ => println!("Created link: {} {} {}", from, link_type, to),
            }
            Ok(())
        }

        LinkCommands::View { link_id } => {
            let mut path_params = HashMap::new();
            path_params.insert("linkId".to_string(), link_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issueLink/{linkId}",
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

        LinkCommands::Delete { link_id, yes } => {
            if !yes
                && !super::confirm_delete_prompt(&format!(
                    "Delete issue link {}? (y/N): ",
                    link_id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("linkId".to_string(), link_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issueLink/{linkId}",
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
                    println!("{}", json!({"id": link_id, "status": "deleted"}));
                }
                _ => println!("Deleted issue link {}", link_id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_create_body() {
        let body = build_create_body("TEAM-1", "TEAM-2", "blocks", None);
        assert_eq!(body["type"]["name"], "blocks");
        assert_eq!(body["outwardIssue"]["key"], "TEAM-1");
        assert_eq!(body["inwardIssue"]["key"], "TEAM-2");
        assert!(body.get("comment").is_none());
    }

    #[test]
    fn test_link_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-123".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}",
            &path_params,
            &[("fields".to_string(), "issuelinks".to_string())],
        );
        assert!(url.contains("/rest/api/3/issue/TEAM-123"));
        assert!(url.contains("fields=issuelinks"));
    }

    #[test]
    fn test_link_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("linkId".to_string(), "10500".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issueLink/{linkId}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issueLink/10500"
        );
    }
}
