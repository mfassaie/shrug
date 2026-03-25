//! Jira issue watcher sub-entity: list, add, remove watchers.

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

/// Watcher subcommands.
#[derive(Subcommand)]
pub enum WatcherCommands {
    /// List watchers on an issue
    List {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
    },
    /// Add a watcher to an issue (defaults to current user)
    Create {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Account ID of the user to add (defaults to @me)
        #[arg(long)]
        user: Option<String>,
    },
    /// Remove a watcher from an issue
    Delete {
        /// Issue key (e.g., TEAM-123)
        issue_key: String,
        /// Account ID of the user to remove (or @me)
        #[arg(long)]
        user: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Build the watcher create body: a JSON string (not an object).
pub fn build_create_body(account_id: &str) -> Value {
    Value::String(account_id.to_string())
}

/// Execute a watcher command.
pub fn execute(
    cmd: &WatcherCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
) -> Result<(), ShrugError> {
    match cmd {
        WatcherCommands::List { issue_key } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/watchers",
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

        WatcherCommands::Create { issue_key, user } => {
            let resolved = super::resolve_at_me(
                user.as_deref().or(Some("@me")),
                client,
                credential,
                base_url,
            )?;
            let account_id = resolved.ok_or_else(|| {
                ShrugError::UsageError("Could not resolve user account ID.".into())
            })?;

            let request_body = build_create_body(&account_id);

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/watchers",
                &path_params,
                &[],
            );

            http::execute_request(
                client,
                Method::POST,
                &url,
                Some(credential),
                Some(&request_body),
                &[],
            )?;

            match output_format {
                OutputFormat::Json => {
                    println!("{}", json!({"accountId": account_id, "status": "added"}));
                }
                _ => println!("Added watcher {} to {}", account_id, issue_key),
            }
            Ok(())
        }

        WatcherCommands::Delete {
            issue_key,
            user,
            yes,
        } => {
            if !yes
                && !super::confirm_delete_prompt(&format!(
                    "Remove watcher {} from {}? (y/N): ",
                    user, issue_key
                ))?
            {
                return Ok(());
            }

            let resolved = super::resolve_at_me(
                Some(user.as_str()),
                client,
                credential,
                base_url,
            )?;
            let account_id = resolved.ok_or_else(|| {
                ShrugError::UsageError("Could not resolve user account ID.".into())
            })?;

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
            let url = http::build_url(
                base_url,
                "/rest/api/3/issue/{issueIdOrKey}/watchers",
                &path_params,
                &[("accountId".to_string(), account_id.clone())],
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
                    println!("{}", json!({"accountId": account_id, "status": "removed"}));
                }
                _ => println!("Removed watcher {} from {}", account_id, issue_key),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_create_body() {
        let body = build_create_body("5b10ac8d82e05b22cc7d4ef5");
        // The body must be a JSON string, not an object
        assert!(body.is_string());
        assert_eq!(body.as_str().unwrap(), "5b10ac8d82e05b22cc7d4ef5");
    }

    #[test]
    fn test_watcher_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-123".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/watchers",
            &path_params,
            &[("accountId".to_string(), "abc123".to_string())],
        );
        assert!(url.contains("/rest/api/3/issue/TEAM-123/watchers"));
        assert!(url.contains("accountId=abc123"));
    }

    #[test]
    fn test_watcher_list_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-456".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}/watchers",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-456/watchers"
        );
    }
}
