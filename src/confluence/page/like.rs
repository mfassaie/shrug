//! Confluence content like sub-entity: view (count), list (users). Read-only.
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

/// Like subcommands.
#[derive(Subcommand)]
pub enum LikeCommands {
    /// View like count on content
    View {
        /// Content ID (page or blogpost)
        content_id: String,
    },
    /// List users who liked the content
    List {
        /// Content ID (page or blogpost)
        content_id: String,
    },
}

/// Execute a like command.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cmd: &LikeCommands,
    credential: &ResolvedCredential,
    client: &Client,
    base_url: &str,
    output_format: &OutputFormat,
    color_enabled: bool,
    parent_type: &str,
) -> Result<(), ShrugError> {
    match cmd {
        LikeCommands::View { content_id } => {
            let url = format!(
                "{}/wiki/api/v2/{}/{}/likes/count",
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

        LikeCommands::List { content_id } => {
            let url = format!(
                "{}/wiki/api/v2/{}/{}/likes/users",
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
    }
}
