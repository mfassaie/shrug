use clap::Subcommand;

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
