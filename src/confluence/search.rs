//! Confluence search entity: CQL search with shorthand flags (v1 API).

use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;

use crate::auth::credentials::ResolvedCredential;
use crate::cli::{ColorChoice, OutputFormat};
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Search entity subcommands.
#[derive(Subcommand)]
pub enum SearchCommands {
    /// Search Confluence content using CQL
    List {
        /// CQL query (positional, overrides --cql and shorthand flags)
        cql_query: Option<String>,
        /// CQL query (overrides shorthand flags)
        #[arg(long)]
        cql: Option<String>,
        /// Filter by space key
        #[arg(long)]
        space: Option<String>,
        /// Filter by content type (page, blogpost, etc.)
        #[arg(long = "type")]
        content_type: Option<String>,
        /// Filter by title (contains match)
        #[arg(short = 't', long)]
        title: Option<String>,
        /// Filter by label
        #[arg(short = 'l', long)]
        label: Option<String>,
        /// Filter by contributor (accountId, @me for currentUser())
        #[arg(long)]
        contributor: Option<String>,
        /// Filter by ancestor page ID
        #[arg(long)]
        ancestor: Option<String>,
        /// Filter by last modified date (e.g., 2024-01-01, -7d)
        #[arg(long)]
        updated: Option<String>,
        /// CQL ORDER BY clause
        #[arg(long)]
        order_by: Option<String>,
        /// Expand additional data
        #[arg(long)]
        expand: Option<String>,
        /// Include archived content
        #[arg(long)]
        include_archived: bool,
    },
}

// ---------------------------------------------------------------------------
// CQL builder
// ---------------------------------------------------------------------------

/// Build a CQL query string from shorthand flags.
///
/// Uses `~` (contains) for title and `=` (exact match) for other fields.
/// Contributor value `@me` is translated to `currentUser()`.
#[allow(clippy::too_many_arguments)]
pub fn build_cql(
    space: Option<&str>,
    content_type: Option<&str>,
    title: Option<&str>,
    label: Option<&str>,
    contributor: Option<&str>,
    ancestor: Option<&str>,
    updated: Option<&str>,
    order_by: Option<&str>,
) -> Option<String> {
    let mut clauses: Vec<String> = Vec::new();

    if let Some(s) = space {
        clauses.push(format!("space = \"{}\"", s));
    }
    if let Some(t) = content_type {
        clauses.push(format!("type = \"{}\"", t));
    }
    if let Some(t) = title {
        clauses.push(format!("title ~ \"{}\"", t));
    }
    if let Some(l) = label {
        clauses.push(format!("label = \"{}\"", l));
    }
    if let Some(c) = contributor {
        if c.eq_ignore_ascii_case("@me") {
            clauses.push("contributor = currentUser()".to_string());
        } else {
            clauses.push(format!("contributor = \"{}\"", c));
        }
    }
    if let Some(a) = ancestor {
        clauses.push(format!("ancestor = {}", a));
    }
    if let Some(u) = updated {
        clauses.push(format!("lastModified >= \"{}\"", u));
    }

    if clauses.is_empty() && order_by.is_none() {
        return None;
    }

    let mut result = clauses.join(" AND ");

    if let Some(order) = order_by {
        if result.is_empty() {
            result = format!("ORDER BY {}", order);
        } else {
            result = format!("{} ORDER BY {}", result, order);
        }
    }

    Some(result)
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a search command.
pub fn execute(
    cmd: &SearchCommands,
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
        SearchCommands::List {
            cql_query,
            cql,
            space,
            content_type,
            title,
            label,
            contributor,
            ancestor,
            updated,
            order_by,
            expand,
            include_archived,
        } => {
            // Precedence: positional cql_query > --cql > shorthand flags
            let resolved_cql = if let Some(ref q) = cql_query {
                let trimmed = q.trim();
                if !trimmed.is_empty() {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            } else if let Some(ref q) = cql {
                let trimmed = q.trim();
                if !trimmed.is_empty() {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            } else {
                build_cql(
                    space.as_deref(),
                    content_type.as_deref(),
                    title.as_deref(),
                    label.as_deref(),
                    contributor.as_deref(),
                    ancestor.as_deref(),
                    updated.as_deref(),
                    order_by.as_deref(),
                )
            };

            let mut base_params: Vec<(String, String)> = Vec::new();
            if let Some(ref q) = resolved_cql {
                base_params.push(("cql".to_string(), q.clone()));
            }
            if let Some(ref e) = expand {
                base_params.push(("expand".to_string(), e.clone()));
            }
            if *include_archived {
                base_params.push(("includeArchivedSpaces".to_string(), "true".to_string()));
            }

            let url_base = http::build_url(
                &base_url, "/wiki/rest/api/search", &HashMap::new(), &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url_base, None);
                return Ok(());
            }

            // Confluence v1 search uses `start`/`limit` params (not startAt/maxResults)
            let page_size: u32 = 25;
            let effective_limit = limit.unwrap_or(u32::MAX) as usize;
            let mut all_results: Vec<serde_json::Value> = Vec::new();
            let mut start: u64 = 0;

            loop {
                let mut page_params = base_params.clone();
                page_params.push(("start".to_string(), start.to_string()));
                page_params.push(("limit".to_string(), page_size.to_string()));

                let url = http::build_url(&url_base, "", &std::collections::HashMap::new(), &page_params);
                let result = http::execute_request(
                    client, Method::GET, &url, Some(credential), None, &[],
                )?;

                let json_val = match result {
                    Some(v) => v,
                    None => break,
                };

                let page_results = crate::core::pagination::extract_results(&json_val)
                    .cloned().unwrap_or_default();
                let count = page_results.len() as u32;

                if count == 0 { break; }
                all_results.extend(page_results);

                if all_results.len() >= effective_limit {
                    all_results.truncate(effective_limit);
                    break;
                }

                // Confluence v1 search uses `totalSize` or link-based pagination
                if !crate::core::pagination::has_more_link(&json_val) {
                    // Fallback: check if we got a full page
                    if count < page_size { break; }
                }

                start += count as u64;
            }

            let json_val = serde_json::Value::Array(all_results);
            if !json_val.as_array().is_none_or(|a| a.is_empty()) {
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
    fn test_search_cql_from_shorthand() {
        let cql = build_cql(
            Some("DOCS"),
            Some("page"),
            Some("meeting notes"),
            Some("important"),
            Some("@me"),
            None,
            None,
            None,
        );
        let q = cql.unwrap();
        assert!(q.contains("space = \"DOCS\""));
        assert!(q.contains("type = \"page\""));
        // Title uses ~ (contains), not =
        assert!(q.contains("title ~ \"meeting notes\""));
        assert!(q.contains("label = \"important\""));
        assert!(q.contains("contributor = currentUser()"));
    }

    #[test]
    fn test_search_positional_overrides_flags() {
        // Simulate the precedence logic from execute
        let positional = Some("type = blogpost");
        let cql_flag = Some("type = page");

        let resolved = if let Some(q) = positional {
            let trimmed = q.trim();
            if !trimmed.is_empty() {
                Some(trimmed.to_string())
            } else {
                None
            }
        } else if let Some(q) = cql_flag {
            let trimmed = q.trim();
            if !trimmed.is_empty() {
                Some(trimmed.to_string())
            } else {
                None
            }
        } else {
            None
        };

        assert_eq!(resolved.unwrap(), "type = blogpost");
    }

    #[test]
    fn test_search_url() {
        let query_params = vec![
            ("cql".to_string(), "type = page".to_string()),
        ];
        let url = http::build_url(
            "https://site.atlassian.net",
            "/wiki/rest/api/search",
            &HashMap::new(),
            &query_params,
        );
        assert!(url.contains("/wiki/rest/api/search"));
        // v1 path, not v2
        assert!(!url.contains("/wiki/api/v2/"));
    }
}
