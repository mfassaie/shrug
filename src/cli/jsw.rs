use clap::Subcommand;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum JswCommands {
    /// Board operations (list, create, view, delete)
    Board {
        #[command(subcommand)]
        command: crate::jsw::board::BoardCommands,
    },
    /// Sprint operations (list, create, view, edit, delete)
    Sprint {
        #[command(subcommand)]
        command: crate::jsw::sprint::SprintCommands,
    },
    /// Epic agile operations (view, edit, list issues)
    Epic {
        #[command(subcommand)]
        command: crate::jsw::epic::EpicCommands,
    },
}
