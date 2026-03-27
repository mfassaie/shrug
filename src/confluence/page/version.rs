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
                let display_val = if matches!(output_format, OutputFormat::Json) {
                    json_val.clone()
                } else if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                    output::project_array(results, &[
                        ("Number", "/number"),
                        ("Message", "/message"),
                        ("Created", "/createdAt"),
                        ("Author", "/authorId"),
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
                let display_val = if matches!(output_format, OutputFormat::Json) {
                    json_val.clone()
                } else {
                    output::project(json_val, &[
                        ("Number", "/number"),
                        ("Message", "/message"),
                        ("Created", "/createdAt"),
                        ("Author", "/authorId"),
                        ("Minor Edit", "/minorEdit"),
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
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_version_list_url() {
        let url = format!(
            "{}/wiki/api/v2/{}/{}/versions",
            "https://site.atlassian.net", "pages", "12345"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/pages/12345/versions"
        );
    }

    #[test]
    fn test_version_view_url() {
        let url = format!(
            "{}/wiki/api/v2/{}/{}/versions/{}",
            "https://site.atlassian.net", "pages", "12345", "3"
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/api/v2/pages/12345/versions/3"
        );
    }

    #[test]
    fn test_version_list_url_blogpost() {
        let url = format!(
            "{}/wiki/api/v2/{}/{}/versions",
            "https://site.atlassian.net", "blogposts", "67890"
        );
        assert!(url.contains("/wiki/api/v2/blogposts/67890/versions"));
    }
}
