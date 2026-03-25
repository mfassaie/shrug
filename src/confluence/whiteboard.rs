//! Confluence whiteboard entity: create, view, delete operations (v2 API).

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

/// Whiteboard entity subcommands.
#[derive(Subcommand)]
pub enum WhiteboardCommands {
    /// Create a new whiteboard
    Create {
        /// Whiteboard title
        #[arg(short = 't', long)]
        title: String,
        /// Space ID (required)
        #[arg(long)]
        space_id: String,
        /// Parent content ID
        #[arg(long)]
        parent_id: Option<String>,
    },
    /// View a whiteboard
    View {
        /// Whiteboard ID
        id: String,
    },
    /// Delete a whiteboard
    Delete {
        /// Whiteboard ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for whiteboard creation.
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

/// Execute a whiteboard command.
pub fn execute(
    cmd: &WhiteboardCommands,
    credential: &ResolvedCredential,
    client: &Client,
    output_format: &OutputFormat,
    color: &ColorChoice,
    _limit: Option<u32>,
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
        WhiteboardCommands::Create {
            title,
            space_id,
            parent_id,
        } => {
            let request_body = build_create_body(title, space_id, parent_id.as_deref());

            let url = format!("{}/wiki/api/v2/whiteboards", base_url);
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
                            println!("Created whiteboard {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        WhiteboardCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/whiteboards/{id}",
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

        WhiteboardCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete whiteboard {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                &base_url,
                "/wiki/api/v2/whiteboards/{id}",
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
                _ => println!("Deleted whiteboard {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whiteboard_create_body() {
        let body = build_create_body("Design Session", "12345", Some("67890"));
        assert_eq!(body["title"], "Design Session");
        assert_eq!(body["spaceId"], "12345");
        assert_eq!(body["parentId"], "67890");
    }
}
