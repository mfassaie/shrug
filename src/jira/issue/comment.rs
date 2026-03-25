//! Jira issue comment sub-entity: LCRUD operations.

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

/// Comment subcommands.
#[derive(Subcommand)]
pub enum CommentCommands {
    /// List comments on an issue
    List {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
    },
    /// Add a comment to an issue
    Create {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Comment body in markdown
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read comment body from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Visibility restriction type (e.g., role, group)
        #[arg(long)]
        visibility_type: Option<String>,
        /// Visibility restriction value (e.g., Developers)
        #[arg(long)]
        visibility_value: Option<String>,
    },
    /// View a specific comment
    View {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Comment ID
        comment_id: String,
    },
    /// Edit a comment
    Edit {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Comment ID
        comment_id: String,
        /// New comment body in markdown
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read comment body from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
    },
    /// Delete a comment
    Delete {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Comment ID
        comment_id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Build a comment request body from markdown text with optional visibility.
pub fn build_comment_body(
    text: &str,
    visibility_type: Option<&str>,
    visibility_value: Option<&str>,
) -> Value {
    let adf = markdown_to_adf::markdown_to_adf(text);
    let mut body = json!({ "body": adf });

    if let (Some(vtype), Some(vvalue)) = (visibility_type, visibility_value) {
        body["visibility"] = json!({
            "type": vtype,
            "value": vvalue,
        });
    }

    body
}

/// Execute a comment command.
pub fn execute(
    cmd: &CommentCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        CommentCommands::List { issue_key } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/comment",
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

        CommentCommands::Create {
            issue_key,
            body,
            body_file,
            visibility_type,
            visibility_value,
        } => {
            let description = super::read_description(body.as_deref(), body_file.as_deref())?;
            let adf = description.ok_or_else(|| {
                ShrugError::UsageError(
                    "Comment body is required. Provide --body or --body-file.".into(),
                )
            })?;

            let mut request_body = json!({ "body": adf });
            if let (Some(vtype), Some(vvalue)) = (visibility_type.as_deref(), visibility_value.as_deref()) {
                request_body["visibility"] = json!({
                    "type": vtype,
                    "value": vvalue,
                });
            }

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/comment",
                &path_params,
                &[],
            );

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
                            println!("Created comment {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        CommentCommands::View {
            issue_key,
            comment_id,
        } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("id".to_string(), comment_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/comment/{id}",
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

        CommentCommands::Edit {
            issue_key,
            comment_id,
            body,
            body_file,
        } => {
            let description = super::read_description(body.as_deref(), body_file.as_deref())?;
            let adf = description.ok_or_else(|| {
                ShrugError::UsageError(
                    "Comment body is required. Provide --body or --body-file.".into(),
                )
            })?;

            let request_body = json!({ "body": adf });

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("id".to_string(), comment_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/comment/{id}",
                &path_params,
                &[],
            );

            let result = http::execute_request(
                client,
                Method::PUT,
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
                        println!("Updated comment {}", comment_id);
                    }
                }
            }
            Ok(())
        }

        CommentCommands::Delete {
            issue_key,
            comment_id,
            yes,
        } => {
            if !yes
                && !super::confirm_delete_prompt(&format!(
                    "Delete comment {} on {}? (y/N): ",
                    comment_id, issue_key
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("id".to_string(), comment_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/comment/{id}",
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
                    println!("{}", json!({"id": comment_id, "status": "deleted"}));
                }
                _ => println!("Deleted comment {}", comment_id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_create_body() {
        let body = build_comment_body("Hello world", None, None);
        assert!(body.get("body").is_some());
        let adf = &body["body"];
        assert_eq!(adf["type"], "doc");
        assert!(body.get("visibility").is_none());
    }

    #[test]
    fn test_comment_create_with_visibility() {
        let body = build_comment_body("Secret note", Some("role"), Some("Developers"));
        assert!(body.get("body").is_some());
        let vis = &body["visibility"];
        assert_eq!(vis["type"], "role");
        assert_eq!(vis["value"], "Developers");
    }

    #[test]
    fn test_comment_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-123".to_string());
        path_params.insert("id".to_string(), "10042".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/comment/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-123/comment/10042"
        );
    }

    #[test]
    fn test_comment_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-456".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/comment",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-456/comment"
        );
    }

    #[test]
    fn test_comment_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-789".to_string());
        path_params.insert("id".to_string(), "20001".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/comment/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-789/comment/20001"
        );
    }

    #[test]
    fn test_comment_edit_body() {
        // Edit only sends body (no visibility), same ADF structure
        let body = build_comment_body("Updated text", None, None);
        assert!(body.get("body").is_some());
        assert_eq!(body["body"]["type"], "doc");
        assert!(body.get("visibility").is_none());
    }
}
