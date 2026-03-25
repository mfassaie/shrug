//! Confluence blogpost entity: LCRUD operations (v2 API).

// Blogpost sub-entities reuse page sub-entity modules with parent_type="blogposts"
use crate::confluence::page;

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

/// Blog post entity subcommands.
#[derive(Subcommand)]
pub enum BlogpostCommands {
    /// List blog posts
    List {
        /// Filter by space ID
        #[arg(long)]
        space_id: Option<String>,
        /// Filter by title
        #[arg(short = 't', long)]
        title: Option<String>,
        /// Post status (current, draft)
        #[arg(long)]
        status: Option<String>,
        /// Sort order (e.g., -modified-date, title)
        #[arg(long)]
        order_by: Option<String>,
    },
    /// Create a new blog post
    Create {
        /// Blog post title
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
        /// Post status (current, draft)
        #[arg(long)]
        status: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// View a blog post
    View {
        /// Blog post ID
        id: String,
    },
    /// Edit a blog post (auto-increments version)
    Edit {
        /// Blog post ID
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
        /// Post status (current, draft)
        #[arg(long)]
        status: Option<String>,
        /// Version message
        #[arg(long)]
        version_message: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// Delete a blog post
    Delete {
        /// Blog post ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Comment operations on a blog post
    Comment {
        #[command(subcommand)]
        command: page::comment::CommentCommands,
    },
    /// Attachment operations on a blog post
    Attachment {
        #[command(subcommand)]
        command: page::attachment::AttachmentCommands,
    },
    /// Label operations on a blog post
    Label {
        #[command(subcommand)]
        command: page::label::LabelCommands,
    },
    /// Property operations on a blog post
    Property {
        #[command(subcommand)]
        command: page::property::PropertyCommands,
    },
    /// Version history of a blog post
    Version {
        #[command(subcommand)]
        command: page::version::VersionCommands,
    },
    /// Like information on a blog post
    Like {
        #[command(subcommand)]
        command: page::like::LikeCommands,
    },
    /// Content restrictions on a blog post
    Restriction {
        #[command(subcommand)]
        command: page::restriction::RestrictionCommands,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Read body content from --body or --body-file. Returns raw storage format text.
fn read_body_content(
    body: Option<&str>,
    body_file: Option<&str>,
) -> Result<Option<String>, ShrugError> {
    if let Some(text) = body {
        return Ok(Some(text.to_string()));
    }
    if let Some(path) = body_file {
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
        return Ok(Some(content));
    }
    Ok(None)
}

/// Build JSON request body for blogpost creation.
pub fn build_create_body(
    title: &str,
    space_id: &str,
    body_content: Option<&str>,
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

    body
}

/// Build JSON request body for blogpost edit (includes version auto-increment).
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

/// Build query parameters for blogpost list.
pub fn build_list_query_params(
    space_id: Option<&str>,
    title: Option<&str>,
    status: Option<&str>,
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
    if let Some(o) = order_by {
        params.push(("sort".to_string(), o.to_string()));
    }
    params
}

/// Fetch the current version number for a blogpost.
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

/// Execute a blogpost command.
pub fn execute(
    cmd: &BlogpostCommands,
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
        BlogpostCommands::List {
            space_id,
            title,
            status,
            order_by,
        } => {
            let mut query_params = build_list_query_params(
                space_id.as_deref(),
                title.as_deref(),
                status.as_deref(),
                order_by.as_deref(),
            );
            if let Some(lim) = limit {
                query_params.push(("limit".to_string(), lim.to_string()));
            }

            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/blogposts",
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

        BlogpostCommands::Create {
            title,
            space_id,
            body,
            body_file,
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
                    status.as_deref(),
                )
            };

            let url = format!("{}/wiki/api/v2/blogposts", base_url);
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
                            println!("Created blogpost {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        BlogpostCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/blogposts/{id}",
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

        BlogpostCommands::Edit {
            id,
            title,
            body,
            body_file,
            status,
            version_message,
            from_json,
        } => {
            // Fetch current version number (required for edit)
            let view_url = format!("{}/wiki/api/v2/blogposts/{}", base_url, id);
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
                    status.as_deref(),
                    next_version,
                    version_message.as_deref(),
                )
            };

            let url = format!("{}/wiki/api/v2/blogposts/{}", base_url, id);
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
                _ => println!("Updated blogpost {} (version {})", id, next_version),
            }
            Ok(())
        }

        BlogpostCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete blogpost {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/blogposts/{id}",
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
                _ => println!("Deleted blogpost {}", id),
            }
            Ok(())
        }

        BlogpostCommands::Comment { command } => {
            page::comment::execute(command, credential, client, &base_url, output_format, color_enabled, "blogposts")
        }
        BlogpostCommands::Attachment { command } => {
            page::attachment::execute(command, credential, client, &base_url, output_format, color_enabled, "blogposts")
        }
        BlogpostCommands::Label { command } => {
            page::label::execute(command, credential, client, &base_url, output_format, color_enabled, "blogposts")
        }
        BlogpostCommands::Property { command } => {
            page::property::execute(command, credential, client, &base_url, output_format, color_enabled, "blogposts")
        }
        BlogpostCommands::Version { command } => {
            page::version::execute(command, credential, client, &base_url, output_format, color_enabled, "blogposts")
        }
        BlogpostCommands::Like { command } => {
            page::like::execute(command, credential, client, &base_url, output_format, color_enabled, "blogposts")
        }
        BlogpostCommands::Restriction { command } => {
            page::restriction::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blogpost_create_body() {
        let body = build_create_body(
            "Release Notes",
            "12345",
            Some("<p>Version 2.0 is here</p>"),
            Some("current"),
        );
        assert_eq!(body["title"], "Release Notes");
        assert_eq!(body["spaceId"], "12345");
        assert_eq!(body["body"]["representation"], "storage");
        assert_eq!(body["body"]["value"], "<p>Version 2.0 is here</p>");
        assert_eq!(body["status"], "current");
        // No parentId for blogposts
        assert!(body.get("parentId").is_none());
    }

    #[test]
    fn test_blogpost_list_url() {
        let query_params = build_list_query_params(
            Some("12345"),
            None,
            Some("current"),
            None,
        );
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/blogposts",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/wiki/api/v2/blogposts"));
        assert!(url.contains("space-id=12345"));
        assert!(url.contains("status=current"));
    }
}
