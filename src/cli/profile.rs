use clap::Subcommand;

use crate::auth::profile::AuthType;

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Create a new profile
    Create {
        /// Profile name (lowercase, alphanumeric, hyphens)
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
    View {
        /// Profile name
        name: String,
    },

    /// Set a profile as the default
    Use {
        /// Profile name
        name: String,
    },

    /// Update an existing profile
    Update {
        /// Profile name
        name: String,

        /// New site URL
        #[arg(long)]
        site: Option<String>,

        /// New email address
        #[arg(long)]
        email: Option<String>,

        /// New authentication type
        #[arg(long, value_enum)]
        auth_type: Option<AuthType>,
    },

    /// Delete a profile
    Delete {
        /// Profile name
        name: String,
    },
}
