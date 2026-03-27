//! Jira issue remote link sub-entity: LCRUD operations.

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

/// Remote link subcommands.
#[derive(Subcommand)]
pub enum RemoteLinkCommands {
    /// List remote links on an issue
    List {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
    },
    /// Create a remote link on an issue
    Create {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// URL of the remote link
        #[arg(long)]
        url: String,
        /// Title of the remote link
        #[arg(short = 't', long)]
        title: String,
        /// Summary text
        #[arg(long)]
        summary: Option<String>,
        /// Relationship description (e.g., "relates to")
        #[arg(long)]
        relationship: Option<String>,
        /// Global ID for the remote link
        #[arg(long)]
        global_id: Option<String>,
    },
    /// View a specific remote link
    View {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Remote link ID
        link_id: String,
    },
    /// Edit a remote link
    Edit {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Remote link ID
        link_id: String,
        /// Updated URL
        #[arg(long)]
        url: Option<String>,
        /// Updated title
        #[arg(short = 't', long)]
        title: Option<String>,
        /// Updated summary
        #[arg(long)]
        summary: Option<String>,
        /// Updated relationship
        #[arg(long)]
        relationship: Option<String>,
    },
    /// Delete a remote link
    Delete {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Remote link ID
        link_id: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Build a remote link create request body.
pub fn build_create_body(
    url: &str,
    title: &str,
    summary: Option<&str>,
    relationship: Option<&str>,
    global_id: Option<&str>,
) -> Value {
    let mut object = json!({
        "url": url,
        "title": title,
    });
    if let Some(s) = summary {
        object["summary"] = json!(s);
    }

    let mut payload = json!({ "object": object });
    if let Some(r) = relationship {
        payload["relationship"] = json!(r);
    }
    if let Some(g) = global_id {
        payload["globalId"] = json!(g);
    }

    payload
}

/// Build a remote link edit request body. All fields are optional.
pub fn build_edit_body(
    url: Option<&str>,
    title: Option<&str>,
    summary: Option<&str>,
    relationship: Option<&str>,
) -> Value {
    let mut object = json!({});
    if let Some(u) = url {
        object["url"] = json!(u);
    }
    if let Some(t) = title {
        object["title"] = json!(t);
    }
    if let Some(s) = summary {
        object["summary"] = json!(s);
    }

    let mut payload = json!({ "object": object });
    if let Some(r) = relationship {
        payload["relationship"] = json!(r);
    }

    payload
}

/// Execute a remote link command.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cmd: &RemoteLinkCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        RemoteLinkCommands::List { issue_key } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/remotelink",
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
                    output::project_array(
                        json_val.as_array().unwrap_or(&vec![]),
                        &[
                            ("ID", "/id"),
                            ("Title", "/object/title"),
                            ("URL", "/object/url"),
                        ],
                    )
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

        RemoteLinkCommands::Create {
            issue_key,
            url: link_url,
            title,
            summary,
            relationship,
            global_id,
        } => {
            let request_body = build_create_body(
                link_url,
                title,
                summary.as_deref(),
                relationship.as_deref(),
                global_id.as_deref(),
            );

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/remotelink",
                &path_params,
                &[],
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
                        if let Some(id) = json_val.get("id").and_then(|v| v.as_u64()) {
                            println!("Created remote link {}", id);
                        } else if let Some(id) = json_val.get("id").and_then(|v| v.as_str()) {
                            println!("Created remote link {}", id);
                        }
                    }
                }
            }
            Ok(())
        }

        RemoteLinkCommands::View {
            issue_key,
            link_id,
        } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("remoteLinkId".to_string(), link_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/remotelink/{remoteLinkId}",
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
                        ("Title", "/object/title"),
                        ("URL", "/object/url"),
                        ("Relationship", "/relationship"),
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

        RemoteLinkCommands::Edit {
            issue_key,
            link_id,
            url: link_url,
            title,
            summary,
            relationship,
        } => {
            let request_body = build_edit_body(
                link_url.as_deref(),
                title.as_deref(),
                summary.as_deref(),
                relationship.as_deref(),
            );

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("remoteLinkId".to_string(), link_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/remotelink/{remoteLinkId}",
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
                    println!("{}", json!({"id": link_id, "status": "updated"}));
                }
                _ => println!("Updated remote link {}", link_id),
            }
            Ok(())
        }

        RemoteLinkCommands::Delete {
            issue_key,
            link_id,
            yes,
        } => {
            if !yes
                && !super::confirm_delete_prompt(&format!(
                    "Delete remote link {} on {}? (y/N): ",
                    link_id, issue_key
                ))?
            {
                return Ok(());
            }

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            path_params.insert("remoteLinkId".to_string(), link_id.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/remotelink/{remoteLinkId}",
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
                    println!("{}", json!({"id": link_id, "status": "deleted"}));
                }
                _ => println!("Deleted remote link {}", link_id),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_link_create_body() {
        let body = build_create_body(
            "https://example.com",
            "Example Link",
            Some("A test link"),
            None,
            None,
        );
        assert_eq!(body["object"]["url"], "https://example.com");
        assert_eq!(body["object"]["title"], "Example Link");
        assert_eq!(body["object"]["summary"], "A test link");
        assert!(body.get("relationship").is_none());
    }

    #[test]
    fn test_remote_link_edit_body() {
        let body = build_edit_body(
            Some("https://updated.com"),
            None,
            None,
            Some("causes"),
        );
        assert_eq!(body["object"]["url"], "https://updated.com");
        assert!(body["object"].get("title").is_none());
        assert_eq!(body["relationship"], "causes");
    }

    #[test]
    fn test_remote_link_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-123".to_string());
        path_params.insert("remoteLinkId".to_string(), "10001".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/remotelink/{remoteLinkId}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-123/remotelink/10001"
        );
    }

    #[test]
    fn test_remote_link_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-456".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/remotelink",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-456/remotelink"
        );
    }

    #[test]
    fn test_remote_link_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-789".to_string());
        path_params.insert("remoteLinkId".to_string(), "20002".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/remotelink/{remoteLinkId}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-789/remotelink/20002"
        );
    }
}
