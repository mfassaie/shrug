//! Confluence smart link (embed) entity: create, view, delete operations (v2 API).

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

/// Smart link (embed) entity subcommands.
#[derive(Subcommand)]
pub enum SmartLinkCommands {
    /// Create a new smart link (embed)
    Create {
        /// URL to embed
        url: String,
        /// Space ID (required)
        #[arg(long)]
        space_id: String,
        /// Parent content ID
        #[arg(long)]
        parent_id: Option<String>,
        /// Smart link title
        #[arg(short = 't', long)]
        title: Option<String>,
    },
    /// View a smart link (embed)
    View {
        /// Smart link ID
        id: String,
    },
    /// Delete a smart link (embed)
    Delete {
        /// Smart link ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JSON request body for smart link creation.
pub fn build_create_body(
    url: &str,
    space_id: &str,
    parent_id: Option<&str>,
    title: Option<&str>,
) -> Value {
    let mut body = json!({
        "url": url,
        "spaceId": space_id,
    });

    if let Some(pid) = parent_id {
        body["parentId"] = json!(pid);
    }

    if let Some(t) = title {
        body["title"] = json!(t);
    }

    body
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a smart link command.
pub fn execute(
    cmd: &SmartLinkCommands,
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
        SmartLinkCommands::Create {
            url,
            space_id,
            parent_id,
            title,
        } => {
            let request_body = build_create_body(
                url,
                space_id,
                parent_id.as_deref(),
                title.as_deref(),
            );

            let api_url = format!("{}/wiki/api/v2/embeds", base_url);
            let result = http::execute_request(
                client,
                Method::POST,
                &api_url,
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
                            println!("Created smart link {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        SmartLinkCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let api_url = http::build_url(
                &base_url,
                "/wiki/api/v2/embeds/{id}",
                &path_params,
                &[],
            );

            let result = http::execute_request(
                client,
                Method::GET,
                &api_url,
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

        SmartLinkCommands::Delete { id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete smart link {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let api_url = http::build_url(
                &base_url,
                "/wiki/api/v2/embeds/{id}",
                &path_params,
                &[],
            );

            http::execute_request(
                client,
                Method::DELETE,
                &api_url,
                Some(credential),
                None,
                &[],
            )?;

            match output_format {
                OutputFormat::Json => {
                    println!("{}", json!({"id": id, "status": "deleted"}));
                }
                _ => println!("Deleted smart link {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {}
