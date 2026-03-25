//! Confluence content label sub-entity: list, create, delete.
//!
//! Shared between pages and blogposts via the `parent_type` parameter.
//! List uses v2; create and delete use v1.

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::json;

use crate::auth::credentials::ResolvedCredential;
use crate::cli::OutputFormat;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Label subcommands.
#[derive(Subcommand)]
pub enum LabelCommands {
    /// List labels on content
    List {
        /// Content ID (page or blogpost)
        content_id: String,
    },
    /// Add a label to content
    Create {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Label name
        name: String,
    },
    /// Remove a label from content
    Delete {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Label name to remove
        label_name: String,
    },
}

/// Execute a label command.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cmd: &LabelCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
    parent_type: &str,
) -> Result<(), ShrugError> {
    match cmd {
        LabelCommands::List { content_id } => {
            let url = format!(
                "{}/wiki/api/v2/{}/{}/labels",
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

        LabelCommands::Create { content_id, name } => {
            // v1 API expects an array of label objects
            let request_body = json!([
                {
                    "prefix": "global",
                    "name": name,
                }
            ]);

            let url = format!(
                "{}/wiki/rest/api/content/{}/label",
                base_url, content_id
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
                    _ => println!("Added label \"{}\" to content {}", name, content_id),
                }
            }
            Ok(())
        }

        LabelCommands::Delete {
            content_id,
            label_name,
        } => {
            // v1 API with label name in path. No confirmation for lightweight operation.
            let url = format!(
                "{}/wiki/rest/api/content/{}/label/{}",
                base_url, content_id, label_name
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
                        json!({"contentId": content_id, "label": label_name, "status": "deleted"})
                    );
                }
                _ => println!("Removed label \"{}\" from content {}", label_name, content_id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_create_body() {
        let body = json!([
            {
                "prefix": "global",
                "name": "reviewed",
            }
        ]);
        assert!(body.is_array());
        let arr = body.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["prefix"], "global");
        assert_eq!(arr[0]["name"], "reviewed");
    }

    #[test]
    fn test_label_delete_url() {
        let url = format!(
            "{}/wiki/rest/api/content/{}/label/{}",
            "https://site.atlassian.net", "12345", "reviewed"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/rest/api/content/12345/label/reviewed"
        );
    }
}
