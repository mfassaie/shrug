//! Jira audit log entity: list-only operations.

use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;

use crate::auth::credentials::ResolvedCredential;
use crate::cli::{ColorChoice, OutputFormat};
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Audit log entity subcommands.
#[derive(Subcommand)]
pub enum AuditCommands {
    /// List audit log records
    List {
        /// Filter audit records (keyword search)
        #[arg(long)]
        filter: Option<String>,
        /// From date (ISO 8601)
        #[arg(long)]
        from: Option<String>,
        /// To date (ISO 8601)
        #[arg(long)]
        to: Option<String>,
    },
}

/// Build query parameters for audit list.
pub fn build_list_query_params(
    filter: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(f) = filter {
        params.push(("filter".to_string(), f.to_string()));
    }
    if let Some(fr) = from {
        params.push(("from".to_string(), fr.to_string()));
    }
    if let Some(t) = to {
        params.push(("to".to_string(), t.to_string()));
    }
    params
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute an audit command.
pub fn execute(
    cmd: &AuditCommands,
    credential: &ResolvedCredential,
    client: &Client,
    output_format: &OutputFormat,
    color: &ColorChoice,
    limit: Option<u32>,
    dry_run: bool,
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
        AuditCommands::List { filter, from, to } => {
            let query_params = build_list_query_params(
                filter.as_deref(),
                from.as_deref(),
                to.as_deref(),
            );
            let url_base = http::build_url(
                &base_url, "/rest/api/3/auditing/record", &HashMap::new(), &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url_base, None);
                return Ok(());
            }

            let results = http::execute_paginated_get(
                client, &url_base, credential, &query_params, &[], limit, 50, false,
            )?;
            if !results.is_empty() {
                let json_val = if matches!(output_format, OutputFormat::Json) {
                    serde_json::Value::Array(results)
                } else {
                    output::project_array(&results, &[
                        ("ID", "/id"),
                        ("Summary", "/summary"),
                        ("Category", "/category"),
                        ("Created", "/created"),
                    ])
                };
                let formatted = output::format_response(
                    &json_val.to_string(), output_format,
                    is_terminal::is_terminal(std::io::stdout()), color_enabled, None,
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
    fn test_audit_list_url() {
        let query_params = build_list_query_params(
            Some("user created"),
            Some("2025-01-01"),
            Some("2025-03-01"),
        );
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/auditing/record",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/rest/api/3/auditing/record"));
        assert!(url.contains("filter=user+created"));
        assert!(url.contains("from=2025-01-01"));
        assert!(url.contains("to=2025-03-01"));
    }
}
