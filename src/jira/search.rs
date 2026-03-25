//! Jira search entity: JQL search with shorthand flags.

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::auth::credentials::ResolvedCredential;
use crate::cli::{ColorChoice, OutputFormat};
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Search entity subcommands.
#[derive(Subcommand)]
pub enum SearchCommands {
    /// Search issues using JQL
    List {
        /// JQL query (positional, overrides --jql and shorthand flags)
        jql_query: Option<String>,
        /// JQL query (overrides shorthand flags)
        #[arg(long)]
        jql: Option<String>,
        /// Filter by project key
        #[arg(long)]
        project: Option<String>,
        /// Filter by assignee (use @me for current user)
        #[arg(short = 'a', long)]
        assignee: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by issue type
        #[arg(long = "type")]
        issue_type: Option<String>,
        /// Filter by priority
        #[arg(long)]
        priority: Option<String>,
        /// Filter by label
        #[arg(short = 'l', long)]
        label: Option<String>,
        /// Filter by last updated (e.g., -7d, -1w)
        #[arg(long)]
        updated: Option<String>,
        /// JQL ORDER BY clause (e.g., "priority DESC")
        #[arg(long)]
        order_by: Option<String>,
        /// Comma-separated list of fields to return
        #[arg(long, value_delimiter = ',')]
        fields: Option<Vec<String>>,
        /// Expand additional data (e.g., changelog, renderedFields)
        #[arg(long)]
        expand: Option<String>,
    },
}

/// Build the search request body from resolved JQL, limit, fields, and expand.
pub fn build_search_body(
    jql: Option<&str>,
    limit: Option<u32>,
    fields: Option<&[String]>,
    expand: Option<&str>,
) -> Value {
    let mut body = json!({});

    if let Some(q) = jql {
        body["jql"] = json!(q);
    }
    if let Some(lim) = limit {
        body["maxResults"] = json!(lim);
    }
    if let Some(f) = fields {
        body["fields"] = json!(f);
    }
    if let Some(e) = expand {
        body["expand"] = json!(e.split(',').collect::<Vec<_>>());
    }

    body
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a search command.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cmd: &SearchCommands,
    credential: &ResolvedCredential,
    client: &Client,
    output_format: &OutputFormat,
    color: &ColorChoice,
    limit: Option<u32>,
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
        SearchCommands::List {
            jql_query,
            jql,
            project,
            assignee,
            status,
            issue_type,
            priority,
            label,
            updated,
            order_by,
            fields,
            expand,
        } => {
            // Precedence: positional jql_query > --jql > shorthand flags
            let resolved_jql = if jql_query.is_some() {
                jql_query.as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string())
            } else if jql.is_some() {
                jql.as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string())
            } else {
                crate::jira::issue::build_list_jql(
                    None,
                    project.as_deref(),
                    assignee.as_deref(),
                    status.as_deref(),
                    issue_type.as_deref(),
                    priority.as_deref(),
                    label.as_deref(),
                    updated.as_deref(),
                    order_by.as_deref(),
                )
            };

            let request_body = build_search_body(
                resolved_jql.as_deref(),
                limit,
                fields.as_deref(),
                expand.as_deref(),
            );

            let url = format!("{}/rest/api/3/search", base_url);
            let result = http::execute_request(
                client,
                Method::POST,
                &url,
                Some(credential),
                Some(&request_body),
                &[],
            )?;

            if let Some(ref json_val) = result {
                let formatted = output::format_response(
                    &json_val.to_string(),
                    output_format,
                    is_terminal::is_terminal(std::io::stdout()),
                    color_enabled,
                    fields.as_deref(),
                );
                println!("{}", formatted);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_jql_from_shorthand() {
        // When no positional or --jql is given, shorthand flags are used
        let jql = crate::jira::issue::build_list_jql(
            None,
            Some("TEAM"),
            Some("@me"),
            Some("Open"),
            None,
            None,
            None,
            None,
            None,
        );
        let q = jql.unwrap();
        assert!(q.contains("project = \"TEAM\""));
        assert!(q.contains("assignee = currentUser()"));
        assert!(q.contains("status = \"Open\""));
    }

    #[test]
    fn test_search_positional_overrides_flags() {
        // Positional jql_query takes priority over everything
        let positional = Some("status = Done");
        let jql_flag = Some("status = Open");

        // Simulate the precedence logic
        let resolved = if positional.is_some() {
            positional.filter(|s| !s.trim().is_empty()).map(|s| s.to_string())
        } else if jql_flag.is_some() {
            jql_flag.filter(|s| !s.trim().is_empty()).map(|s| s.to_string())
        } else {
            None
        };

        assert_eq!(resolved.unwrap(), "status = Done");
    }
}
