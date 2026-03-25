//! Jira label entity: list-only operations.

use std::collections::HashMap;

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;

use crate::auth::credentials::ResolvedCredential;
use crate::cli::{ColorChoice, OutputFormat};
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Label entity subcommands.
#[derive(Subcommand)]
pub enum LabelCommands {
    /// List all labels
    List {},
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Execute a label command.
pub fn execute(
    cmd: &LabelCommands,
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
        LabelCommands::List {} => {
            let mut query_params: Vec<(String, String)> = Vec::new();
            if let Some(lim) = limit {
                query_params.push(("maxResults".to_string(), lim.to_string()));
            }

            let url = http::build_url(
                &base_url,
                "/rest/api/3/label",
                &HashMap::new(),
                &query_params,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_list_url() {
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/label",
            &HashMap::new(),
            &[],
        );
        assert_eq!(url, "https://site.atlassian.net/rest/api/3/label");
    }
}
