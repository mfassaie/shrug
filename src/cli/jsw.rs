use clap::Subcommand;

#[derive(Subcommand)]
pub enum JswCommands {
    /// Accepts any arguments (temporary — entity subcommands added in Phase 6)
    #[command(external_subcommand)]
    External(Vec<String>),
}
