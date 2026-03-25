//! Confluence content restriction sub-entity: view, edit, delete.
//!
//! Uses v1 API paths that are content-generic (no parent_type needed).

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::auth::credentials::ResolvedCredential;
use crate::cli::OutputFormat;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Restriction subcommands.
#[derive(Subcommand)]
pub enum RestrictionCommands {
    /// View restrictions on content
    View {
        /// Content ID
        content_id: String,
    },
    /// Set restrictions on content
    Edit {
        /// Content ID
        content_id: String,
        /// Operation type (read or update)
        operation: String,
        /// Account IDs of users to restrict to (repeatable)
        #[arg(long)]
        user: Vec<String>,
        /// Group names to restrict to (repeatable)
        #[arg(long)]
        group: Vec<String>,
    },
    /// Remove all restrictions from content
    Delete {
        /// Content ID
        content_id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Build the restriction request body for the v1 API.
pub fn build_restriction_body(
    operation: &str,
    users: &[String],
    groups: &[String],
) -> Value {
    let user_results: Vec<Value> = users
        .iter()
        .map(|u| json!({"accountId": u}))
        .collect();

    let group_results: Vec<Value> = groups
        .iter()
        .map(|g| json!({"name": g}))
        .collect();

    json!([
        {
            "operation": operation,
            "restrictions": {
                "user": {
                    "results": user_results,
                },
                "group": {
                    "results": group_results,
                },
            },
        }
    ])
}

/// Execute a restriction command.
pub fn execute(
    cmd: &RestrictionCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        RestrictionCommands::View { content_id } => {
            let url = format!(
                "{}/wiki/rest/api/content/{}/restriction",
                base_url, content_id
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

        RestrictionCommands::Edit {
            content_id,
            operation,
            user,
            group,
        } => {
            let request_body = build_restriction_body(operation, user, group);

            let url = format!(
                "{}/wiki/rest/api/content/{}/restriction",
                base_url, content_id
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
                    println!(
                        "{}",
                        json!({"contentId": content_id, "operation": operation, "status": "updated"})
                    );
                }
                _ => println!(
                    "Set {} restriction on content {}",
                    operation, content_id
                ),
            }
            Ok(())
        }

        RestrictionCommands::Delete { content_id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Remove all restrictions from content {}? (y/N): ",
                    content_id
                ))?
            {
                return Ok(());
            }

            let url = format!(
                "{}/wiki/rest/api/content/{}/restriction",
                base_url, content_id
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
                    println!(
                        "{}",
                        json!({"contentId": content_id, "status": "restrictions_removed"})
                    );
                }
                _ => println!("Removed all restrictions from content {}", content_id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restriction_edit_body() {
        let body = build_restriction_body(
            "read",
            &["user-abc-123".to_string()],
            &["developers".to_string()],
        );
        assert!(body.is_array());
        let arr = body.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["operation"], "read");

        let users = arr[0]["restrictions"]["user"]["results"]
            .as_array()
            .unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["accountId"], "user-abc-123");

        let groups = arr[0]["restrictions"]["group"]["results"]
            .as_array()
            .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0]["name"], "developers");
    }

    #[test]
    fn test_restriction_url() {
        let url = format!(
            "{}/wiki/rest/api/content/{}/restriction",
            "https://site.atlassian.net", "12345"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/rest/api/content/12345/restriction"
        );
    }
}
