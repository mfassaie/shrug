use clap::Subcommand;

#[derive(Subcommand)]
pub enum JiraCommands {
    /// Accepts any arguments (temporary — entity subcommands added in Phase 5)
    #[command(external_subcommand)]
    External(Vec<String>),
}
