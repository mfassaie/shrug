use clap::Subcommand;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ConfluenceCommands {
    /// Space operations
    Space {
        #[command(subcommand)]
        command: crate::confluence::space::SpaceCommands,
    },
    /// Page operations
    Page {
        #[command(subcommand)]
        command: crate::confluence::page::PageCommands,
    },
    /// Blog post operations
    #[command(name = "blogpost")]
    Blogpost {
        #[command(subcommand)]
        command: crate::confluence::blogpost::BlogpostCommands,
    },
    /// Whiteboard operations
    Whiteboard {
        #[command(subcommand)]
        command: crate::confluence::whiteboard::WhiteboardCommands,
    },
    /// Database operations
    Database {
        #[command(subcommand)]
        command: crate::confluence::database::DatabaseCommands,
    },
    /// Folder operations
    Folder {
        #[command(subcommand)]
        command: crate::confluence::folder::FolderCommands,
    },
    /// Custom content operations
    #[command(name = "custom-content")]
    CustomContent {
        #[command(subcommand)]
        command: crate::confluence::custom_content::CustomContentCommands,
    },
    /// Smart link (embed) operations
    #[command(name = "smart-link")]
    SmartLink {
        #[command(subcommand)]
        command: crate::confluence::smart_link::SmartLinkCommands,
    },
    /// Task operations
    Task {
        #[command(subcommand)]
        command: crate::confluence::task::TaskCommands,
    },
    /// Search with CQL
    Search {
        #[command(subcommand)]
        command: crate::confluence::search::SearchCommands,
    },
}
