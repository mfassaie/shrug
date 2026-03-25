//! Jira issue attachment sub-entity: list, create (upload), view, delete.
//!
//! Attachment upload uses multipart form data with a manual request build
//! (not the shared `execute_request` helper) due to the multipart body
//! and the required `X-Atlassian-Token: no-check` header.

use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::auth::credentials::{AuthScheme, ResolvedCredential};
use crate::cli::OutputFormat;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Attachment subcommands.
#[derive(Subcommand)]
pub enum AttachmentCommands {
    /// List attachments on an issue
    List {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
    },
    /// Upload a file attachment to an issue
    Create {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Path to the file to upload
        #[arg(short = 'f', long)]
        file: String,
    },
    /// View attachment metadata
    View {
        /// Attachment ID
        id: String,
    },
    /// Delete an attachment
    Delete {
        /// Attachment ID
        id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Execute an attachment command.
pub fn execute(
    cmd: &AttachmentCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        AttachmentCommands::List { issue_key } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}",
                &path_params,
                &[("fields".to_string(), "attachment".to_string())],
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
                let attachments = json_val
                    .get("fields")
                    .and_then(|f| f.get("attachment"))
                    .cloned()
                    .unwrap_or(Value::Array(vec![]));
                let formatted = output::format_response(
                    &attachments.to_string(),
                    output_format,
                    is_terminal::is_terminal(std::io::stdout()),
                    color_enabled,
                    None,
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        AttachmentCommands::Create { issue_key, file } => {
            let file_path = std::path::Path::new(file);
            if !file_path.exists() {
                return Err(ShrugError::UsageError(format!(
                    "File not found: {}",
                    file
                )));
            }

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/attachments",
                &path_params,
                &[],
            );

            let max_attempts = 3;
            let mut last_err: Option<ShrugError> = None;

            for attempt in 0..max_attempts {
                // Rebuild the multipart form each attempt (Form is not cloneable)
                let form = multipart::Form::new()
                    .file("file", file_path)
                    .map_err(|e| {
                        ShrugError::UsageError(format!("Failed to read file {}: {}", file, e))
                    })?;

                let req = client
                    .post(&url)
                    .multipart(form)
                    .header("X-Atlassian-Token", "no-check");

                let req = match &credential.scheme {
                    AuthScheme::Basic { email, api_token } => {
                        req.basic_auth(email, Some(api_token))
                    }
                    AuthScheme::Bearer { access_token } => req.bearer_auth(access_token),
                };

                match req.send() {
                    Ok(response) => {
                        let status = response.status().as_u16();

                        if (200..300).contains(&status) {
                            let text = response.text().map_err(ShrugError::NetworkError)?;
                            if !text.is_empty() {
                                let json_val: Value = serde_json::from_str(&text)
                                    .unwrap_or_else(|_| Value::String(text));
                                match output_format {
                                    OutputFormat::Json => {
                                        println!(
                                            "{}",
                                            serde_json::to_string_pretty(&json_val)
                                                .unwrap_or_default()
                                        );
                                    }
                                    _ => {
                                        if let Some(arr) = json_val.as_array() {
                                            for att in arr {
                                                if let Some(id) =
                                                    att.get("id").and_then(|v| v.as_str())
                                                {
                                                    println!("Created attachment {}", id);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            return Ok(());
                        }

                        if http::is_retryable_status(status) && attempt < max_attempts - 1 {
                            let delay = std::time::Duration::from_secs(
                                if attempt == 0 { 1 } else { 3 },
                            );
                            tracing::info!(attempt = attempt + 1, "Retrying attachment upload");
                            std::thread::sleep(delay);
                            let error_body = response.text().unwrap_or_default();
                            last_err = Some(http::map_status_to_error(status, error_body));
                            continue;
                        }

                        let error_body = response.text().unwrap_or_default();
                        return Err(http::map_status_to_error(status, error_body));
                    }
                    Err(e) => {
                        if (e.is_timeout() || e.is_connect()) && attempt < max_attempts - 1 {
                            let delay = std::time::Duration::from_secs(
                                if attempt == 0 { 1 } else { 3 },
                            );
                            tracing::info!(attempt = attempt + 1, "Retrying after network error");
                            std::thread::sleep(delay);
                            last_err = Some(ShrugError::NetworkError(e));
                            continue;
                        }
                        return Err(ShrugError::NetworkError(e));
                    }
                }
            }

            Err(last_err.unwrap_or_else(|| ShrugError::ServerError {
                status: 0,
                message: "Upload failed after retries".into(),
            }))
        }

        AttachmentCommands::View { id } => {
            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/attachment/{id}",
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

        AttachmentCommands::Delete { id, yes } => {
            if !yes
                && !super::confirm_delete_prompt(&format!(
                    "Delete attachment {}? (y/N): ",
                    id
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("id".to_string(), id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/attachment/{id}",
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
                _ => println!("Deleted attachment {}", id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::core::http;

    #[test]
    fn test_attachment_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-123".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}",
            &path_params,
            &[("fields".to_string(), "attachment".to_string())],
        );
        assert!(url.contains("/rest/api/3/issue/TEAM-123"));
        assert!(url.contains("fields=attachment"));
    }

    #[test]
    fn test_attachment_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "99001".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/attachment/{id}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/attachment/99001"
        );
    }
}
