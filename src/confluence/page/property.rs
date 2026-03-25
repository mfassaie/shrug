//! Confluence content property sub-entity: list, create, view, edit, delete.
//!
//! Shared between pages and blogposts via the `parent_type` parameter.
//! All operations use the v2 API.

use std::fs;
use std::io::{self, Read};

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::auth::credentials::ResolvedCredential;
use crate::cli::OutputFormat;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Property subcommands.
#[derive(Subcommand)]
pub enum PropertyCommands {
    /// List properties on content
    List {
        /// Content ID (page or blogpost)
        content_id: String,
    },
    /// Create a property on content
    Create {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Property key
        #[arg(short = 'k', long)]
        key: String,
        /// Property value (must be valid JSON)
        #[arg(long, conflicts_with = "value_file")]
        value: Option<String>,
        /// Read value from file (must contain valid JSON)
        #[arg(long, conflicts_with = "value")]
        value_file: Option<String>,
    },
    /// View a specific property
    View {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Property ID
        property_id: String,
    },
    /// Edit a property (auto-increments version)
    Edit {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Property ID
        property_id: String,
        /// Property value (must be valid JSON)
        #[arg(long, conflicts_with = "value_file")]
        value: Option<String>,
        /// Read value from file (must contain valid JSON)
        #[arg(long, conflicts_with = "value")]
        value_file: Option<String>,
    },
    /// Delete a property
    Delete {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Property ID
        property_id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Read and parse JSON value from --value or --value-file.
fn read_json_value(
    value: Option<&str>,
    value_file: Option<&str>,
) -> Result<Value, ShrugError> {
    let raw = if let Some(v) = value {
        v.to_string()
    } else if let Some(path) = value_file {
        if path == "-" {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).map_err(|e| {
                ShrugError::UsageError(format!("Failed to read from stdin: {}", e))
            })?;
            buf
        } else {
            fs::read_to_string(path).map_err(|e| {
                ShrugError::UsageError(format!("Failed to read {}: {}", path, e))
            })?
        }
    } else {
        return Err(ShrugError::UsageError(
            "Property value is required (use --value or --value-file)".into(),
        ));
    };

    serde_json::from_str(&raw).map_err(|e| {
        ShrugError::UsageError(format!("Invalid JSON value: {}", e))
    })
}

/// Fetch a property to extract its current version number.
fn fetch_property_version(
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
                        "Could not determine current property version from API response".into(),
                    )
                })
        }
        None => Err(ShrugError::UsageError(
            "Empty response when fetching property version".into(),
        )),
    }
}

/// Execute a property command.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cmd: &PropertyCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
    parent_type: &str,
) -> Result<(), ShrugError> {
    match cmd {
        PropertyCommands::List { content_id } => {
            let url = format!(
                "{}/wiki/api/v2/{}/{}/properties",
                base_url, parent_type, content_id
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

        PropertyCommands::Create {
            content_id,
            key,
            value,
            value_file,
        } => {
            let json_value = read_json_value(value.as_deref(), value_file.as_deref())?;

            let request_body = json!({
                "key": key,
                "value": json_value,
            });

            let url = format!(
                "{}/wiki/api/v2/{}/{}/properties",
                base_url, parent_type, content_id
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
                            println!("Created property {} with key \"{}\"", id, key);
                        } else {
                            println!("Created property with key \"{}\"", key);
                        }
                    }
                }
            }
            Ok(())
        }

        PropertyCommands::View {
            content_id,
            property_id,
        } => {
            let url = format!(
                "{}/wiki/api/v2/{}/{}/properties/{}",
                base_url, parent_type, content_id, property_id
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

        PropertyCommands::Edit {
            content_id,
            property_id,
            value,
            value_file,
        } => {
            let json_value = read_json_value(value.as_deref(), value_file.as_deref())?;

            // Fetch current version for auto-increment
            let view_url = format!(
                "{}/wiki/api/v2/{}/{}/properties/{}",
                base_url, parent_type, content_id, property_id
            );
            let current_version =
                fetch_property_version(client, credential, &view_url)?;
            let next_version = current_version + 1;

            let request_body = json!({
                "value": json_value,
                "version": {
                    "number": next_version,
                },
            });

            let url = format!(
                "{}/wiki/api/v2/{}/{}/properties/{}",
                base_url, parent_type, content_id, property_id
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
                        json!({"id": property_id, "status": "updated", "version": next_version})
                    );
                }
                _ => println!(
                    "Updated property {} (version {})",
                    property_id, next_version
                ),
            }
            Ok(())
        }

        PropertyCommands::Delete {
            content_id,
            property_id,
            yes,
        } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete property {}? (y/N): ",
                    property_id
                ))?
            {
                return Ok(());
            }

            let url = format!(
                "{}/wiki/api/v2/{}/{}/properties/{}",
                base_url, parent_type, content_id, property_id
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
                    println!("{}", json!({"id": property_id, "status": "deleted"}));
                }
                _ => println!("Deleted property {}", property_id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_create_body() {
        let json_value: Value = serde_json::from_str(r#"{"enabled":true}"#).unwrap();
        let body = json!({
            "key": "app.config",
            "value": json_value,
        });
        assert_eq!(body["key"], "app.config");
        assert_eq!(body["value"]["enabled"], true);
    }

    #[test]
    fn test_property_url() {
        let url = format!(
            "{}/wiki/api/v2/{}/{}/properties",
            "https://site.atlassian.net", "pages", "12345"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/pages/12345/properties"
        );

        let view_url = format!(
            "{}/wiki/api/v2/{}/{}/properties/{}",
            "https://site.atlassian.net", "blogposts", "67890", "prop-1"
        );
        assert!(view_url.contains("/wiki/api/v2/blogposts/67890/properties/prop-1"));
    }
}
