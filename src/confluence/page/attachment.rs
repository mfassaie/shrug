//! Confluence content attachment sub-entity: list, create, view, edit, delete.
//!
//! Shared between pages and blogposts via the `parent_type` parameter.
//! Create and edit use v1 multipart upload; list, view, delete use v2.

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
    /// List attachments on content
    List {
        /// Content ID (page or blogpost)
        content_id: String,
    },
    /// Upload a file attachment to content
    Create {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Path to the file to upload
        #[arg(short = 'f', long)]
        file: String,
        /// Attachment comment
        #[arg(long)]
        comment: Option<String>,
    },
    /// View attachment metadata
    View {
        /// Attachment ID
        attachment_id: String,
    },
    /// Update an existing attachment's file
    Edit {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Attachment ID
        attachment_id: String,
        /// Path to the new file
        #[arg(short = 'f', long)]
        file: String,
        /// Attachment comment
        #[arg(long)]
        comment: Option<String>,
    },
    /// Delete an attachment
    Delete {
        /// Attachment ID
        attachment_id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Execute an attachment command.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cmd: &AttachmentCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
    parent_type: &str,
) -> Result<(), ShrugError> {
    match cmd {
        AttachmentCommands::List { content_id } => {
            let url = format!(
                "{}/wiki/api/v2/{}/{}/attachments",
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
                let display_val = if matches!(output_format, OutputFormat::Json) {
                    json_val.clone()
                } else if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                    output::project_array(results, &[
                        ("ID", "/id"),
                        ("Title", "/title"),
                        ("Media Type", "/mediaType"),
                        ("File Size", "/fileSize"),
                    ])
                } else {
                    json_val.clone()
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

        AttachmentCommands::Create {
            content_id,
            file,
            comment,
        } => {
            let file_path = std::path::Path::new(file);
            if !file_path.exists() {
                return Err(ShrugError::UsageError(format!(
                    "File not found: {}",
                    file
                )));
            }

            // v1 multipart upload
            let url = format!(
                "{}/wiki/rest/api/content/{}/child/attachment",
                base_url, content_id
            );

            let max_attempts = 3;
            let mut last_err: Option<ShrugError> = None;

            for attempt in 0..max_attempts {
                // Rebuild the multipart form each attempt (Form is not cloneable)
                let mut form = multipart::Form::new()
                    .file("file", file_path)
                    .map_err(|e| {
                        ShrugError::UsageError(format!("Failed to read file {}: {}", file, e))
                    })?;

                if let Some(ref c) = comment {
                    form = form.text("comment", c.clone());
                }

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
                                        if let Some(results) =
                                            json_val.get("results").and_then(|r| r.as_array())
                                        {
                                            for att in results {
                                                if let Some(id) =
                                                    att.get("id").and_then(|v| v.as_str())
                                                {
                                                    println!("Created attachment {}", id);
                                                }
                                            }
                                        } else if let Some(id) =
                                            json_val.get("id").and_then(|v| v.as_str())
                                        {
                                            println!("Created attachment {}", id);
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

        AttachmentCommands::View { attachment_id } => {
            let url = format!(
                "{}/wiki/api/v2/attachments/{}",
                base_url, attachment_id
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
                        ("Media Type", "/mediaType"),
                        ("File Size", "/fileSize"),
                        ("Created", "/createdAt"),
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

        AttachmentCommands::Edit {
            content_id,
            attachment_id,
            file,
            comment,
        } => {
            let file_path = std::path::Path::new(file);
            if !file_path.exists() {
                return Err(ShrugError::UsageError(format!(
                    "File not found: {}",
                    file
                )));
            }

            // v1 multipart upload for update
            let url = format!(
                "{}/wiki/rest/api/content/{}/child/attachment/{}/data",
                base_url, content_id, attachment_id
            );

            let mut form = multipart::Form::new()
                .file("file", file_path)
                .map_err(|e| {
                    ShrugError::UsageError(format!("Failed to read file {}: {}", file, e))
                })?;

            if let Some(ref c) = comment {
                form = form.text("comment", c.clone());
            }

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

            let response = req.send().map_err(ShrugError::NetworkError)?;
            let status = response.status().as_u16();

            if !(200..300).contains(&status) {
                let error_body = response.text().unwrap_or_default();
                return Err(ShrugError::ServerError {
                    status,
                    message: error_body,
                });
            }

            match output_format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        json!({"contentId": content_id, "attachmentId": attachment_id, "status": "updated"})
                    );
                }
                _ => println!("Updated attachment {} on content {}", attachment_id, content_id),
            }
            Ok(())
        }

        AttachmentCommands::Delete { attachment_id, yes } => {
            if !yes
                && !crate::jira::issue::confirm_delete_prompt(&format!(
                    "Delete attachment {}? (y/N): ",
                    attachment_id
                ))?
            {
                return Ok(());
            }

            let url = format!(
                "{}/wiki/api/v2/attachments/{}",
                base_url, attachment_id
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
                    println!("{}", json!({"id": attachment_id, "status": "deleted"}));
                }
                _ => println!("Deleted attachment {}", attachment_id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_attachment_list_url_v2() {
        let url = format!(
            "{}/wiki/api/v2/{}/{}/attachments",
            "https://site.atlassian.net", "pages", "12345"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/pages/12345/attachments"
        );

        let bp_url = format!(
            "{}/wiki/api/v2/{}/{}/attachments",
            "https://site.atlassian.net", "blogposts", "67890"
        );
        assert!(bp_url.contains("/wiki/api/v2/blogposts/67890/attachments"));
    }

    #[test]
    fn test_attachment_create_url_v1() {
        let url = format!(
            "{}/wiki/rest/api/content/{}/child/attachment",
            "https://site.atlassian.net", "12345"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/rest/api/content/12345/child/attachment"
        );
    }

    #[test]
    fn test_attachment_view_url_v2() {
        let url = format!(
            "{}/wiki/api/v2/attachments/{}",
            "https://site.atlassian.net", "att-99001"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/attachments/att-99001"
        );
    }

    #[test]
    fn test_attachment_delete_url_v2() {
        let url = format!(
            "{}/wiki/api/v2/attachments/{}",
            "https://site.atlassian.net", "att-88001"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/attachments/att-88001"
        );
    }
}
