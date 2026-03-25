//! Confluence content comment sub-entity: list, create, view, edit, delete.
//!
//! Shared between pages and blogposts via the `parent_type` parameter.

use std::fs;
use std::io::{self, Read};

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::json;

use crate::auth::credentials::ResolvedCredential;
use crate::cli::OutputFormat;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Comment subcommands.
#[derive(Subcommand)]
pub enum CommentCommands {
    /// List comments on content
    List {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Comment type: footer (default) or inline
        #[arg(long = "type")]
        comment_type: Option<String>,
    },
    /// Create a comment on content
    Create {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Comment body in storage format
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read body from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Comment type: footer (default) or inline
        #[arg(long = "type")]
        comment_type: Option<String>,
    },
    /// View a comment
    View {
        /// Comment ID
        comment_id: String,
        /// Comment type: footer (default) or inline
        #[arg(long = "type")]
        comment_type: Option<String>,
    },
    /// Edit a comment (auto-increments version)
    Edit {
        /// Comment ID
        comment_id: String,
        /// Comment body in storage format
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read body from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Comment type: footer (default) or inline
        #[arg(long = "type")]
        comment_type: Option<String>,
    },
    /// Delete a comment
    Delete {
        /// Comment ID
        comment_id: String,
        /// Comment type: footer (default) or inline
        #[arg(long = "type")]
        comment_type: Option<String>,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Read body content from --body or --body-file. Converts markdown to
/// Confluence storage format (XHTML) before returning.
fn read_body_content(
    body: Option<&str>,
    body_file: Option<&str>,
) -> Result<Option<String>, ShrugError> {
    let raw = if let Some(text) = body {
        Some(text.to_string())
    } else if let Some(path) = body_file {
        let content = if path == "-" {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).map_err(|e| {
                ShrugError::UsageError(format!("Failed to read from stdin: {}", e))
            })?;
            buf
        } else {
            fs::read_to_string(path).map_err(|e| {
                ShrugError::UsageError(format!("Failed to read {}: {}", path, e))
            })?
        };
        Some(content)
    } else {
        None
    };

    Ok(raw.map(|text| crate::content::markdown_to_storage::markdown_to_storage(&text)))
}

/// Resolve comment type string to API path segment.
fn comment_path_segment(comment_type: Option<&str>) -> &str {
    match comment_type.unwrap_or("footer") {
        "inline" => "inline-comments",
        _ => "footer-comments",
    }
}

/// Build the parent ID field name for comment creation based on parent_type.
fn parent_id_field(parent_type: &str) -> &str {
    match parent_type {
        "blogposts" => "blogPostId",
        _ => "pageId",
    }
}

/// Execute a comment command.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cmd: &CommentCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
    parent_type: &str,
) -> Result<(), ShrugError> {
    match cmd {
        CommentCommands::List {
            content_id,
            comment_type,
        } => {
            let segment = comment_path_segment(comment_type.as_deref());
            let url = format!(
                "{}/wiki/api/v2/{}/{}/{}",
                base_url, parent_type, content_id, segment
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
            content_id,
            body,
            body_file,
            comment_type,
        } => {
            let segment = comment_path_segment(comment_type.as_deref());
            let body_text = read_body_content(body.as_deref(), body_file.as_deref())?
                .ok_or_else(|| {
                    ShrugError::UsageError(
                        "Comment body is required (use --body or --body-file)".into(),
                    )
                })?;

            let id_field = parent_id_field(parent_type);
            let request_body = json!({
                id_field: content_id,
                "body": {
                    "representation": "storage",
                    "value": body_text,
                },
            });

            let url = format!("{}/wiki/api/v2/{}", base_url, segment);
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
            comment_id,
            comment_type,
        } => {
            let segment = comment_path_segment(comment_type.as_deref());
            let url = format!("{}/wiki/api/v2/{}/{}", base_url, segment, comment_id);

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
            comment_id,
            body,
            body_file,
            comment_type,
        } => {
            let segment = comment_path_segment(comment_type.as_deref());
            let body_text = read_body_content(body.as_deref(), body_file.as_deref())?
                .ok_or_else(|| {
                    ShrugError::UsageError(
                        "Comment body is required (use --body or --body-file)".into(),
                    )
                })?;

            // Fetch current version for auto-increment
            let view_url = format!("{}/wiki/api/v2/{}/{}", base_url, segment, comment_id);
            let current_version =
                super::fetch_current_version(client, credential, &view_url)?;
            let next_version = current_version + 1;

            let request_body = json!({
                "body": {
                    "representation": "storage",
                    "value": body_text,
                },
                "version": {
                    "number": next_version,
                },
            });

            let url = format!("{}/wiki/api/v2/{}/{}", base_url, segment, comment_id);
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
                        json!({"id": comment_id, "status": "updated", "version": next_version})
                    );
                }
                _ => println!("Updated comment {} (version {})", comment_id, next_version),
            }
            Ok(())
        }

        CommentCommands::Delete {
            comment_id,
            comment_type,
            yes,
        } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete comment {}? (y/N): ",
                    comment_id
                ))?
            {
                return Ok(());
            }

            let segment = comment_path_segment(comment_type.as_deref());
            let url = format!("{}/wiki/api/v2/{}/{}", base_url, segment, comment_id);

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
        let id_field = parent_id_field("pages");
        let body = json!({
            id_field: "12345",
            "body": {
                "representation": "storage",
                "value": "<p>Nice work</p>",
            },
        });
        assert_eq!(body["pageId"], "12345");
        assert_eq!(body["body"]["representation"], "storage");
        assert_eq!(body["body"]["value"], "<p>Nice work</p>");
        assert!(body.get("blogPostId").is_none());

        // Blogpost uses blogPostId
        let bp_field = parent_id_field("blogposts");
        let bp_body = json!({
            bp_field: "67890",
            "body": {
                "representation": "storage",
                "value": "<p>Great post</p>",
            },
        });
        assert_eq!(bp_body["blogPostId"], "67890");
        assert!(bp_body.get("pageId").is_none());
    }

    #[test]
    fn test_comment_footer_url() {
        let segment = comment_path_segment(None);
        assert_eq!(segment, "footer-comments");

        let segment_footer = comment_path_segment(Some("footer"));
        assert_eq!(segment_footer, "footer-comments");

        let segment_inline = comment_path_segment(Some("inline"));
        assert_eq!(segment_inline, "inline-comments");

        // List URL shape
        let url = format!(
            "{}/wiki/api/v2/{}/{}/{}",
            "https://site.atlassian.net", "pages", "12345", segment
        );
        assert!(url.contains("/wiki/api/v2/pages/12345/footer-comments"));
    }

    #[test]
    fn test_comment_view_url() {
        let segment = comment_path_segment(None);
        let url = format!(
            "{}/wiki/api/v2/{}/{}",
            "https://site.atlassian.net", segment, "99001"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/footer-comments/99001"
        );
    }

    #[test]
    fn test_comment_edit_url() {
        let segment = comment_path_segment(Some("inline"));
        let url = format!(
            "{}/wiki/api/v2/{}/{}",
            "https://site.atlassian.net", segment, "88001"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/inline-comments/88001"
        );
    }

    #[test]
    fn test_comment_delete_url() {
        let segment = comment_path_segment(None);
        let url = format!(
            "{}/wiki/api/v2/{}/{}",
            "https://site.atlassian.net", segment, "77001"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/footer-comments/77001"
        );
    }
}
