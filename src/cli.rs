use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::auth::profile::AuthType;

#[derive(Parser)]
#[command(name = "shrug", version, about = "A dynamic CLI for Atlassian Cloud")]
pub struct Cli {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    pub output: OutputFormat,

    /// Color output
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto, global = true)]
    pub color: ColorChoice,

    /// Configuration profile to use
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Dry run — show what would happen without making changes
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// JSON request body for operations that require one
    #[arg(long, global = true)]
    pub json: Option<String>,

    /// Fetch all pages of paginated results
    #[arg(long, global = true)]
    pub page_all: bool,

    /// Maximum number of results to fetch (used with --page-all)
    #[arg(long, global = true)]
    pub limit: Option<u32>,

    /// Select specific fields for table/CSV output (comma-separated)
    #[arg(long, global = true)]
    pub fields: Option<String>,

    /// Disable pager for output
    #[arg(long, global = true)]
    pub no_pager: bool,

    /// Enable trace-level logging (full diagnostic output)
    #[arg(long, global = true)]
    pub trace: bool,

    /// Convert Markdown fields in --json body to ADF before sending
    #[arg(long, global = true)]
    pub markdown: bool,

    /// Raw JQL query string (Jira/Jira Software only)
    #[arg(long, global = true)]
    pub jql: Option<String>,

    /// JQL shorthand: filter by project key
    #[arg(long, global = true)]
    pub project: Option<String>,

    /// JQL shorthand: filter by assignee ("me" for current user)
    #[arg(long, global = true)]
    pub assignee: Option<String>,

    /// JQL shorthand: filter by status
    #[arg(long, global = true)]
    pub status: Option<String>,

    /// JQL shorthand: filter by issue type
    #[arg(long, global = true)]
    pub issue_type: Option<String>,

    /// JQL shorthand: filter by priority
    #[arg(long, global = true)]
    pub priority: Option<String>,

    /// JQL shorthand: filter by label
    #[arg(long, global = true)]
    pub label: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Table,
    Yaml,
    Csv,
    Plain,
}

#[derive(Clone, Debug, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Jira Cloud operations
    Jira {
        /// Arguments passed to the Jira subcommand
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Jira Software operations (boards, sprints, backlogs)
    #[command(name = "jira-software")]
    JiraSoftware {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Confluence operations
    Confluence {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Bitbucket Cloud operations
    Bitbucket {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Jira Service Management operations
    Jsm {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Authentication management (set-token, status)
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Profile management (create, list, show, delete, use)
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Cache management (refresh specs)
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
    /// Generate shell completions (bash, zsh, fish, powershell)
    Completions {
        /// Shell to generate completions for
        shell: String,
        /// Generate dynamic completions with live Atlassian lookups
        #[arg(long)]
        dynamic: bool,
    },
    /// Internal: output completion values for dynamic tab-completion
    #[command(name = "_complete", hide = true)]
    Complete {
        /// Completion type: projects, spaces, issues
        completion_type: String,
        /// Extra arguments (e.g. --project FOO)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Create a new profile
    Create {
        /// Profile name (lowercase, alphanumeric, hyphens)
        #[arg(long)]
        name: String,

        /// Atlassian site URL (e.g., mysite.atlassian.net)
        #[arg(long)]
        site: String,

        /// Email address for authentication
        #[arg(long)]
        email: String,

        /// Authentication type
        #[arg(long, value_enum, default_value_t = AuthType::BasicAuth)]
        auth_type: AuthType,
    },

    /// List all profiles
    List,

    /// Show details of a profile
    Show {
        /// Profile name
        #[arg(long)]
        name: String,
    },

    /// Delete a profile
    Delete {
        /// Profile name
        #[arg(long)]
        name: String,
    },

    /// Set a profile as the default
    Use {
        /// Profile name to set as default
        #[arg(long)]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Download/refresh API specs from Atlassian CDN
    Refresh {
        /// Product to refresh (jira, jira-software, confluence, jsm, bitbucket). All if not specified.
        #[arg(long)]
        product: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Store an API token for a profile
    SetToken {
        /// Profile name (uses default if not specified)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Show credential status for a profile
    Status {
        /// Profile name (uses default if not specified)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Authorize an OAuth 2.0 profile via browser flow
    Login {
        /// Profile name (uses default if not specified)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Interactive setup wizard for first-time configuration
    Setup,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn output_format_all_variants() {
        let variants = [
            OutputFormat::Json,
            OutputFormat::Table,
            OutputFormat::Yaml,
            OutputFormat::Csv,
            OutputFormat::Plain,
        ];
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn color_choice_all_variants() {
        let variants = [
            ColorChoice::Auto,
            ColorChoice::Always,
            ColorChoice::Never,
        ];
        assert_eq!(variants.len(), 3);
    }

    #[test]
    fn cli_parses_version_flag() {
        let result = Cli::try_parse_from(["shrug", "--version"]);
        // --version causes early exit, which clap returns as Err(DisplayVersion)
        assert!(result.is_err());
    }

    #[test]
    fn cli_parses_help_flag() {
        let result = Cli::try_parse_from(["shrug", "--help"]);
        assert!(result.is_err()); // --help causes early exit
    }

    #[test]
    fn cli_parses_global_flags() {
        let cli = Cli::try_parse_from([
            "shrug",
            "--output", "json",
            "--color", "never",
            "--verbose",
            "--dry-run",
        ])
        .unwrap();
        assert_eq!(cli.output, OutputFormat::Json);
        assert_eq!(cli.color, ColorChoice::Never);
        assert_eq!(cli.verbose, 1);
        assert!(cli.dry_run);
    }

    #[test]
    fn cli_parses_jira_subcommand() {
        let cli = Cli::try_parse_from(["shrug", "jira", "issues", "get-issue"]).unwrap();
        match cli.command {
            Some(Commands::Jira { ref args }) => {
                assert_eq!(args, &["issues", "get-issue"]);
            }
            _ => panic!("Expected Jira command"),
        }
    }

    #[test]
    fn cli_parses_profile_create() {
        let cli = Cli::try_parse_from([
            "shrug",
            "profile", "create",
            "--name", "test",
            "--site", "test.atlassian.net",
            "--email", "a@b.com",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Profile {
                command: ProfileCommands::Create { ref name, .. },
            }) => assert_eq!(name, "test"),
            _ => panic!("Expected Profile Create"),
        }
    }

    #[test]
    fn cli_parses_cache_refresh() {
        let cli = Cli::try_parse_from(["shrug", "cache", "refresh"]).unwrap();
        match cli.command {
            Some(Commands::Cache {
                command: CacheCommands::Refresh { product },
            }) => assert!(product.is_none()),
            _ => panic!("Expected Cache Refresh"),
        }
    }
}
