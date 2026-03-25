//! Confluence custom content entity: LCRUD operations (v2 API).

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};

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

/// Custom content entity subcommands.
#[derive(Subcommand)]
pub enum CustomContentCommands {
    /// List custom content
    List {
        /// Custom content type (e.g., ac:my-app:content)
        #[arg(long = "type")]
        content_type: Option<String>,
        /// Filter by space ID
        #[arg(long)]
        space_id: Option<String>,
    },
    /// Create custom content
    Create {
        /// Custom content type (e.g., ac:my-app:content)
        #[arg(long = "type")]
        content_type: String,
        /// Content title
        #[arg(short = 't', long)]
        title: String,
        /// Space ID (required)
        #[arg(long)]
        space_id: String,
        /// Body content in storage format
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read body from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Page ID to attach content to
        #[arg(long)]
        page_id: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// View custom content
    View {
        /// Custom content ID
        id: String,
    },
    /// Edit custom content (auto-increments version)
    Edit {
        /// Custom content ID
        id: String,
        /// New title
        #[arg(short = 't', long)]
        title: Option<String>,
        /// Body content in storage format
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read body from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Version message
        #[arg(long)]
        version_message: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// Delete custom content
    Delete {
        /// Custom content ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

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

/// Build JSON request body for custom content creation.
pub fn build_create_body(
    content_type: &str,
    title: &str,
    space_id: &str,
    body_content: Option<&str>,
    page_id: Option<&str>,
) -> Value {
    let mut body = json!({
        "type": content_type,
        "title": title,
        "spaceId": space_id,
    });

    if let Some(content) = body_content {
        body["body"] = json!({
            "representation": "storage",
            "value": content,
        });
    }

    if let Some(pid) = page_id {
        body["pageId"] = json!(pid);
    }

    body
}

/// Build JSON request body for custom content edit (includes version auto-increment).
pub fn build_edit_body(
    id: &str,
    title: Option<&str>,
    body_content: Option<&str>,
    version_number: u64,
    version_message: Option<&str>,
) -> Value {
    let mut body = json!({
        "id": id,
        "version": {
            "number": version_number,
        },
    });

    if let Some(t) = title {
        body["title"] = json!(t);
    }

    if let Some(content) = body_content {
        body["body"] = json!({
            "representation": "storage",
            "value": content,
        });
    }

    if let Some(msg) = version_message {
        body["version"]["message"] = json!(msg);
    }

    body
}

/// Build query parameters for custom content list.
pub fn build_list_query_params(
    content_type: Option<&str>,
    space_id: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(t) = content_type {
        params.push(("type".to_string(), t.to_string()));
    }
    if let Some(s) = space_id {
        params.push(("space-id".to_string(), s.to_string()));
    }
    params
}

/// Fetch the current version number for custom content.
fn fetch_current_version(
    client: &Client,
    credential: &ResolvedCredential,
    url: &str,
) -> Result<u64, ShrugError> {
    let result = http::execute_request(
        client,
        Method::GET,
        url,
        Some(credential),
        None,
        &[],
    )?;

    match result {
        Some(json_val) => {
            json_val
                .get("version")
                .and_then(|v| v.get("number"))
                .and_then(|n| n.as_u64())
                .ok_or_else(|| {
                    ShrugError::UsageError(
                        "Could not determine current version number from API response".into(),
                    )
                })
        }
        None => Err(ShrugError::UsageError(
            "Empty response when fetching current version".into(),
        )),
    }
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a custom content command.
pub fn execute(
    cmd: &CustomContentCommands,
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
        CustomContentCommands::List {
            content_type,
            space_id,
        } => {
            let query_params = build_list_query_params(
                content_type.as_deref(),
                space_id.as_deref(),
            );
            let url_base = http::build_url(
                &base_url, "/wiki/api/v2/custom-content", &HashMap::new(), &[],
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

        CustomContentCommands::Create {
            content_type,
            title,
            space_id,
            body,
            body_file,
            page_id,
            from_json,
        } => {
            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                read_json_file(path)?
            } else {
                let body_content = read_body_content(body.as_deref(), body_file.as_deref())?;
                build_create_body(
                    content_type,
                    title,
                    space_id,
                    body_content.as_deref(),
                    page_id.as_deref(),
                )
            };

            let url = format!("{}/wiki/api/v2/custom-content", base_url);

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
                            println!("Created custom content {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        CustomContentCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/custom-content/{id}",
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

        CustomContentCommands::Edit {
            id,
            title,
            body,
            body_file,
            version_message,
            from_json,
        } => {
            // Fetch current version number (required for edit)
            let view_url = format!("{}/wiki/api/v2/custom-content/{}", base_url, id);
            let current_version = fetch_current_version(client, credential, &view_url)?;
            let next_version = current_version + 1;

            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, merging version if not present");
                let mut json_body = read_json_file(path)?;
                if json_body.get("version").is_none() {
                    let mut version_obj = json!({"number": next_version});
                    if let Some(ref msg) = version_message {
                        version_obj["message"] = json!(msg);
                    }
                    json_body["version"] = version_obj;
                }
                json_body
            } else {
                let body_content = read_body_content(body.as_deref(), body_file.as_deref())?;
                build_edit_body(
                    id,
                    title.as_deref(),
                    body_content.as_deref(),
                    next_version,
                    version_message.as_deref(),
                )
            };

            let url = format!("{}/wiki/api/v2/custom-content/{}", base_url, id);
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
                    println!("{}", json!({"id": id, "status": "updated", "version": next_version}));
                }
                _ => println!("Updated custom content {} (version {})", id, next_version),
            }
            Ok(())
        }

        CustomContentCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete custom content {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/custom-content/{id}",
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
                _ => println!("Deleted custom content {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_content_create_body() {
        let body = build_create_body(
            "ac:my-app:content",
            "My Custom Item",
            "12345",
            Some("<p>Custom data</p>"),
            Some("67890"),
        );
        assert_eq!(body["type"], "ac:my-app:content");
        assert_eq!(body["title"], "My Custom Item");
        assert_eq!(body["spaceId"], "12345");
        assert_eq!(body["body"]["representation"], "storage");
        assert_eq!(body["body"]["value"], "<p>Custom data</p>");
        assert_eq!(body["pageId"], "67890");
    }
}
