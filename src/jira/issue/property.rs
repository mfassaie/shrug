//! Jira issue property sub-entity: list, view, edit (PUT creates), delete.

use std::collections::HashMap;

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
    /// List all properties on an issue
    List {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
    },
    /// View a specific property value
    View {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Property key
        property_key: String,
    },
    /// Set a property value (creates or updates via PUT)
    Edit {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Property key
        property_key: String,
        /// JSON value to set
        #[arg(long, conflicts_with = "value_file")]
        value: Option<String>,
        /// Read JSON value from file (use - for stdin)
        #[arg(long, conflicts_with = "value")]
        value_file: Option<String>,
    },
    /// Delete a property
    Delete {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Property key
        property_key: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Parse a JSON value string into a serde_json::Value.
pub fn parse_property_value(raw: &str) -> Result<Value, ShrugError> {
    serde_json::from_str(raw)
        .map_err(|e| ShrugError::UsageError(format!("Invalid JSON value: {}", e)))
}

/// Execute a property command.
pub fn execute(
    cmd: &PropertyCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        PropertyCommands::List { issue_key } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/properties",
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

        PropertyCommands::View {
            issue_key,
            property_key,
        } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("propertyKey".to_string(), property_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/properties/{propertyKey}",
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

        PropertyCommands::Edit {
            issue_key,
            property_key,
            value,
            value_file,
        } => {
            let raw = if let Some(ref v) = value {
                v.clone()
            } else if let Some(ref path) = value_file {
                if path == "-" {
                    let mut buf = String::new();
                    std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).map_err(
                        |e| ShrugError::UsageError(format!("Failed to read from stdin: {}", e)),
                    )?;
                    buf
                } else {
                    std::fs::read_to_string(path).map_err(|e| {
                        ShrugError::UsageError(format!("Failed to read {}: {}", path, e))
                    })?
                }
            } else {
                return Err(ShrugError::UsageError(
                    "Property value is required. Provide --value or --value-file.".into(),
                ));
            };

            let request_body = parse_property_value(&raw)?;

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("propertyKey".to_string(), property_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/properties/{propertyKey}",
                &path_params,
                &[],
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
                        json!({"key": property_key, "status": "updated"})
                    );
                }
                _ => println!("Updated property {}", property_key),
            }
            Ok(())
        }

        PropertyCommands::Delete {
            issue_key,
            property_key,
            yes,
        } => {
            if !yes
                && !super::confirm_delete_prompt(&format!(
                    "Delete property {} on {}? (y/N): ",
                    property_key, issue_key
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("propertyKey".to_string(), property_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/properties/{propertyKey}",
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
                    println!(
                        "{}",
                        json!({"key": property_key, "status": "deleted"})
                    );
                }
                _ => println!("Deleted property {}", property_key),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn test_property_edit_sends_raw_value() {
        let val = parse_property_value(r#"{"count": 42, "enabled": true}"#).unwrap();
        // The parsed value is sent directly, not wrapped in any envelope
        assert_eq!(val["count"], 42);
        assert_eq!(val["enabled"], true);
    }

    #[test]
    fn test_property_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-123".to_string());
        path_params.insert("propertyKey".to_string(), "my.custom.prop".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/properties/{propertyKey}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-123/properties/my.custom.prop"
        );
    }

    #[test]
    fn test_property_value_and_value_file_conflict() {
        let result = Cli::try_parse_from([
            "shrug",
            "jira",
            "issue",
            "property",
            "edit",
            "TEAM-123",
            "my.key",
            "--value",
            "{}",
            "--value-file",
            "f.json",
        ]);
        assert!(
            result.is_err(),
            "--value and --value-file should conflict"
        );
    }
}
