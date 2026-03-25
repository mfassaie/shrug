use clap::Subcommand;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum JiraCommands {
    /// Issue operations (create, list, view, edit, delete)
    Issue {
        #[command(subcommand)]
        command: crate::jira::issue::IssueCommands,
    },
    /// Project operations (LCRUD, components, versions)
    Project {
        #[command(subcommand)]
        command: crate::jira::project::ProjectCommands,
    },
    /// Filter operations (saved JQL searches)
    Filter {
        #[command(subcommand)]
        command: crate::jira::filter::FilterCommands,
    },
    /// Dashboard operations
    Dashboard {
        #[command(subcommand)]
        command: crate::jira::dashboard::DashboardCommands,
    },
    /// List all labels
    Label {
        #[command(subcommand)]
        command: crate::jira::label::LabelCommands,
    },
    /// Audit log records
    Audit {
        #[command(subcommand)]
        command: crate::jira::audit::AuditCommands,
    },
    /// Search issues with JQL
    Search {
        #[command(subcommand)]
        command: crate::jira::search::SearchCommands,
    },
    /// Accepts any arguments (temporary — remaining entity subcommands added later)
    #[command(external_subcommand)]
    External(Vec<String>),
}
