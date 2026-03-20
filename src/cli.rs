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

    /// Enable trace-level logging (full diagnostic output)
    #[arg(long, global = true)]
    pub trace: bool,

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
    /// Cache management
    Cache {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Generate shell completions
    Completions {
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
}
