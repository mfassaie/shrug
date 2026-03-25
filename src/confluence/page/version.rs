//! Confluence content version sub-entity: list, view (read-only).
//!
//! Shared between pages and blogposts via the `parent_type` parameter.

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;

use crate::auth::credentials::ResolvedCredential;
use crate::cli::OutputFormat;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Version subcommands.
#[derive(Subcommand)]
pub enum VersionCommands {
    /// List version history
    List {
        /// Content ID (page or blogpost)
        content_id: String,
    },
    /// View a specific version
    View {
        /// Content ID (page or blogpost)
        content_id: String,
        /// Version number
        version_number: String,
    },
}

/// Execute a version command.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cmd: &VersionCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
    parent_type: &str,
) -> Result<(), ShrugError> {
    match cmd {
        VersionCommands::List { content_id } => {
            let url = format!(
                "{}/wiki/api/v2/{}/{}/versions",
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

        VersionCommands::View {
            content_id,
            version_number,
        } => {
            let url = format!(
                "{}/wiki/api/v2/{}/{}/versions/{}",
                base_url, parent_type, content_id, version_number
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
