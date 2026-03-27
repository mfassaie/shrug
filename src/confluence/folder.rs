//! Confluence folder entity: create, view, delete operations (v2 API).

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

/// Folder entity subcommands.
#[derive(Subcommand)]
pub enum FolderCommands {
    /// Create a new folder
    Create {
        /// Folder title
        #[arg(short = 't', long)]
        title: String,
        /// Space ID (required)
        #[arg(long)]
        space_id: String,
        /// Parent content ID
        #[arg(long)]
        parent_id: Option<String>,
    },
    /// View a folder
    View {
        /// Folder ID
        id: String,
    },
    /// Delete a folder
    Delete {
        /// Folder ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for folder creation.
pub fn build_create_body(
    title: &str,
    space_id: &str,
    parent_id: Option<&str>,
) -> Value {
    let mut body = json!({
        "spaceId": space_id,
        "title": title,
    });

    if let Some(pid) = parent_id {
        body["parentId"] = json!(pid);
    }

    body
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a folder command.
pub fn execute(
    cmd: &FolderCommands,
    credential: &ResolvedCredential,
    client: &Client,
    output_format: &OutputFormat,
    color: &ColorChoice,
    _limit: Option<u32>,
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
        FolderCommands::Create {
            title,
            space_id,
            parent_id,
        } => {
            let request_body = build_create_body(title, space_id, parent_id.as_deref());

            let url = format!("{}/wiki/api/v2/folders", base_url);

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
                            println!("Created folder {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        FolderCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/folders/{id}",
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
                        ("Space ID", "/spaceId"),
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

        FolderCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete folder {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/folders/{id}",
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
                _ => println!("Deleted folder {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_folder_create_body() {
        let body = build_create_body("My Folder", "12345", Some("67890"));
        assert_eq!(body["title"], "My Folder");
        assert_eq!(body["spaceId"], "12345");
        assert_eq!(body["parentId"], "67890");
    }

    #[test]
    fn test_folder_create_body_no_parent() {
        let body = build_create_body("Root Folder", "11111", None);
        assert_eq!(body["title"], "Root Folder");
        assert_eq!(body["spaceId"], "11111");
        assert!(body.get("parentId").is_none());
    }

    #[test]
    fn test_folder_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "33333".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/folders/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/folders/33333"
        );
    }

    #[test]
    fn test_folder_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "44444".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/api/v2/folders/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/folders/44444"
        );
    }
}
