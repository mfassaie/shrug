use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConfluenceCommands {
    /// Accepts any arguments (temporary — entity subcommands added in Phase 7)
    #[command(external_subcommand)]
    External(Vec<String>),
}
