//! Confluence page entity: LCRUD operations (v2 API).

pub mod attachment;
pub mod comment;
pub mod label;
pub mod like;
pub mod property;
pub mod restriction;
pub mod version;

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

/// Page entity subcommands.
#[derive(Subcommand)]
pub enum PageCommands {
    /// List pages
    List {
        /// Filter by space ID
        #[arg(long)]
        space_id: Option<String>,
        /// Filter by title
        #[arg(short = 't', long)]
        title: Option<String>,
        /// Page status (current, draft, archived)
        #[arg(long)]
        status: Option<String>,
        /// Filter by parent page ID
        #[arg(long)]
        parent_id: Option<String>,
        /// Sort order (e.g., -modified-date, title)
        #[arg(long)]
        order_by: Option<String>,
    },
    /// Create a new page
    Create {
        /// Page title
        #[arg(short = 't', long)]
        title: String,
        /// Space ID (required)
        #[arg(long)]
        space_id: String,
        /// Body content in Confluence storage format
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read body from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Parent page ID
        #[arg(long)]
        parent_id: Option<String>,
        /// Page status (current, draft)
        #[arg(long)]
        status: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// View a page
    View {
        /// Page ID
        id: String,
    },
    /// Edit a page (auto-increments version)
    Edit {
        /// Page ID
        id: String,
        /// New title
        #[arg(short = 't', long)]
        title: Option<String>,
        /// Body content in Confluence storage format
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read body from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Page status (current, draft)
        #[arg(long)]
        status: Option<String>,
        /// Version message
        #[arg(long)]
        version_message: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// Delete a page
    Delete {
        /// Page ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Comment operations on a page
    Comment {
        #[command(subcommand)]
        command: comment::CommentCommands,
    },
    /// Attachment operations on a page
    Attachment {
        #[command(subcommand)]
        command: attachment::AttachmentCommands,
    },
    /// Label operations on a page
    Label {
        #[command(subcommand)]
        command: label::LabelCommands,
    },
    /// Property operations on a page
    Property {
        #[command(subcommand)]
        command: property::PropertyCommands,
    },
    /// Version history of a page
    Version {
        #[command(subcommand)]
        command: version::VersionCommands,
    },
    /// Like information on a page
    Like {
        #[command(subcommand)]
        command: like::LikeCommands,
    },
    /// Content restrictions on a page
    Restriction {
        #[command(subcommand)]
        command: restriction::RestrictionCommands,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Read body content from --body or --body-file. Converts markdown to
/// Confluence storage format (XHTML) before returning.
pub fn read_body_content(
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

/// Build JSON request body for page creation.
pub fn build_create_body(
    title: &str,
    space_id: &str,
    body_content: Option<&str>,
    parent_id: Option<&str>,
    status: Option<&str>,
) -> Value {
    let mut body = json!({
        "spaceId": space_id,
        "status": status.unwrap_or("current"),
        "title": title,
    });

    if let Some(content) = body_content {
        body["body"] = json!({
            "representation": "storage",
            "value": content,
        });
    }

    if let Some(pid) = parent_id {
        body["parentId"] = json!(pid);
    }

    body
}

/// Build JSON request body for page edit (includes version auto-increment).
pub fn build_edit_body(
    id: &str,
    title: Option<&str>,
    body_content: Option<&str>,
    status: Option<&str>,
    version_number: u64,
    version_message: Option<&str>,
) -> Value {
    let mut body = json!({
        "id": id,
        "status": status.unwrap_or("current"),
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

/// Build query parameters for page list.
pub fn build_list_query_params(
    space_id: Option<&str>,
    title: Option<&str>,
    status: Option<&str>,
    parent_id: Option<&str>,
    order_by: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(s) = space_id {
        params.push(("space-id".to_string(), s.to_string()));
    }
    if let Some(t) = title {
        params.push(("title".to_string(), t.to_string()));
    }
    if let Some(st) = status {
        params.push(("status".to_string(), st.to_string()));
    }
    if let Some(p) = parent_id {
        params.push(("parent-id".to_string(), p.to_string()));
    }
    if let Some(o) = order_by {
        params.push(("sort".to_string(), o.to_string()));
    }
    params
}

/// Fetch the current version number for a page/blogpost/custom-content item.
pub fn fetch_current_version(
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

/// Execute a page command.
pub fn execute(
    cmd: &PageCommands,
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
        PageCommands::List {
            space_id,
            title,
            status,
            parent_id,
            order_by,
        } => {
            let query_params = build_list_query_params(
                space_id.as_deref(),
                title.as_deref(),
                status.as_deref(),
                parent_id.as_deref(),
                order_by.as_deref(),
            );
            let url_base = http::build_url(
                &base_url, "/wiki/api/v2/pages", &HashMap::new(), &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url_base, None);
                return Ok(());
            }

            let results = http::execute_paginated_get(
                client, &url_base, credential, &query_params, &[], limit, 25, true,
            )?;
            if !results.is_empty() {
                let json_val = if matches!(output_format, OutputFormat::Json) {
                    serde_json::Value::Array(results)
                } else {
                    output::project_array(&results, &[
                        ("ID", "/id"),
                        ("Title", "/title"),
                        ("Status", "/status"),
                        ("Space ID", "/spaceId"),
                    ])
                };
                let formatted = output::format_response(
                    &json_val.to_string(), output_format,
                    is_terminal::is_terminal(std::io::stdout()), color_enabled, None,
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        PageCommands::Create {
            title,
            space_id,
            body,
            body_file,
            parent_id,
            status,
            from_json,
        } => {
            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                read_json_file(path)?
            } else {
                let body_content = read_body_content(body.as_deref(), body_file.as_deref())?;
                build_create_body(
                    title,
                    space_id,
                    body_content.as_deref(),
                    parent_id.as_deref(),
                    status.as_deref(),
                )
            };

            let url = format!("{}/wiki/api/v2/pages", base_url);

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
                            println!("Created page {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        PageCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/pages/{id}",
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
                        ("ID", "/id"),
                        ("Title", "/title"),
                        ("Status", "/status"),
                        ("Space ID", "/spaceId"),
                        ("Created", "/createdAt"),
                        ("Version", "/version/number"),
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

        PageCommands::Edit {
            id,
            title,
            body,
            body_file,
            status,
            version_message,
            from_json,
        } => {
            // Fetch current version number (required for edit)
            let view_url = format!("{}/wiki/api/v2/pages/{}", base_url, id);
            let current_version = fetch_current_version(client, credential, &view_url)?;
            let next_version = current_version + 1;

            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, merging version if not present");
                let mut json_body = read_json_file(path)?;
                // Merge version into from-json body if not already present
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
                    status.as_deref(),
                    next_version,
                    version_message.as_deref(),
                )
            };

            let url = format!("{}/wiki/api/v2/pages/{}", base_url, id);
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
                _ => println!("Updated page {} (version {})", id, next_version),
            }
            Ok(())
        }

        PageCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete page {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/pages/{id}",
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
                _ => println!("Deleted page {}", id),
            }
            Ok(())
        }

        PageCommands::Comment { command } => {
            comment::execute(command, credential, client, &base_url, output_format, color_enabled, "pages")
        }
        PageCommands::Attachment { command } => {
            attachment::execute(command, credential, client, &base_url, output_format, color_enabled, "pages")
        }
        PageCommands::Label { command } => {
            label::execute(command, credential, client, &base_url, output_format, color_enabled, "pages")
        }
        PageCommands::Property { command } => {
            property::execute(command, credential, client, &base_url, output_format, color_enabled, "pages")
        }
        PageCommands::Version { command } => {
            version::execute(command, credential, client, &base_url, output_format, color_enabled, "pages")
        }
        PageCommands::Like { command } => {
            like::execute(command, credential, client, &base_url, output_format, color_enabled, "pages")
        }
        PageCommands::Restriction { command } => {
            restriction::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_create_body() {
        let body = build_create_body(
            "My Page",
            "12345",
            Some("<p>Hello world</p>"),
            Some("67890"),
            Some("current"),
        );
        assert_eq!(body["title"], "My Page");
        assert_eq!(body["spaceId"], "12345");
        assert_eq!(body["body"]["representation"], "storage");
        assert_eq!(body["body"]["value"], "<p>Hello world</p>");
        assert_eq!(body["parentId"], "67890");
        assert_eq!(body["status"], "current");
    }

    #[test]
    fn test_page_edit_body() {
        let body = build_edit_body(
            "111",
            Some("Updated Title"),
            Some("<p>New content</p>"),
            Some("current"),
            5,
            Some("Fixed typo"),
        );
        assert_eq!(body["id"], "111");
        assert_eq!(body["title"], "Updated Title");
        assert_eq!(body["body"]["representation"], "storage");
        assert_eq!(body["body"]["value"], "<p>New content</p>");
        assert_eq!(body["version"]["number"], 5);
        assert_eq!(body["version"]["message"], "Fixed typo");
        assert_eq!(body["status"], "current");
    }

    #[test]
    fn test_page_list_url() {
        let query_params = build_list_query_params(
            Some("12345"),
            Some("My Page"),
            Some("current"),
            None,
            None,
        );
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/pages",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/wiki/api/v2/pages"));
        assert!(url.contains("space-id=12345"));
        assert!(url.contains("title=My+Page") || url.contains("title=My%20Page"));
    }

    #[test]
    fn test_page_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "55555".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/pages/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/pages/55555"
        );
    }

    #[test]
    fn test_page_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "66666".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/pages/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/pages/66666"
        );
    }

    #[test]
    fn test_page_create_with_from_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("page.json");
        std::fs::write(
            &path,
            r#"{"title":"Custom Page","spaceId":"12345","status":"current"}"#,
        )
        .unwrap();
        let value = crate::jira::issue::read_json_file(path.to_str().unwrap()).unwrap();
        assert_eq!(value["title"], "Custom Page");
        assert_eq!(value["spaceId"], "12345");
    }
}
