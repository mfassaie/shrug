pub mod auth;
pub mod confluence;
pub mod global;
pub mod jira;
pub mod jsw;
pub mod profile;

pub use auth::AuthCommands;
pub use confluence::ConfluenceCommands;
pub use global::{ColorChoice, OutputFormat};
pub use jira::JiraCommands;
pub use jsw::JswCommands;
pub use profile::ProfileCommands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "shrug", version, about = "A static CLI for Atlassian Cloud")]
pub struct Cli {
    /// Output format
    #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    pub output: OutputFormat,

    /// Color output
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto, global = true)]
    pub color: ColorChoice,

    /// Configuration profile to use
    #[arg(short = 'p', long, global = true)]
    pub profile: Option<String>,

    /// Increase verbosity (-v, -vv, -vvv for trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Dry run — show what would happen without making changes
    #[arg(short = 'n', long, global = true)]
    pub dry_run: bool,

    /// Maximum number of results to fetch (implies pagination)
    #[arg(short = 'L', long, global = true)]
    pub limit: Option<u32>,

    /// Open the resource in a browser instead of printing
    #[arg(short = 'w', long, global = true)]
    pub web: bool,

    /// Suppress non-essential output
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Jira Cloud operations
    #[command(visible_alias = "j")]
    Jira {
        #[command(subcommand)]
        command: JiraCommands,
    },
    /// Jira Software operations (boards, sprints)
    #[command(name = "jira-software", visible_alias = "jsw")]
    JiraSoftware {
        #[command(subcommand)]
        command: JswCommands,
    },
    /// Confluence operations
    #[command(visible_aliases = ["c", "conf"])]
    Confluence {
        #[command(subcommand)]
        command: ConfluenceCommands,
    },
    /// Authentication management (set-token, status)
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Profile management (create, list, get, delete)
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn output_format_all_variants() {
        let variants = [OutputFormat::Json, OutputFormat::Table, OutputFormat::Csv];
        assert_eq!(variants.len(), 3);
    }

    #[test]
    fn color_choice_all_variants() {
        let variants = [ColorChoice::Auto, ColorChoice::Always, ColorChoice::Never];
        assert_eq!(variants.len(), 3);
    }

    #[test]
    fn cli_parses_version_flag() {
        let result = Cli::try_parse_from(["shrug", "--version"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_parses_help_flag() {
        let result = Cli::try_parse_from(["shrug", "--help"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_parses_global_flags() {
        let cli = Cli::try_parse_from([
            "shrug",
            "--output",
            "json",
            "--color",
            "never",
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
    fn cli_parses_short_forms() {
        let cli =
            Cli::try_parse_from(["shrug", "-o", "json", "-p", "prod", "-n", "-L", "50", "-q"])
                .unwrap();
        assert_eq!(cli.output, OutputFormat::Json);
        assert_eq!(cli.profile, Some("prod".to_string()));
        assert!(cli.dry_run);
        assert_eq!(cli.limit, Some(50));
        assert!(cli.quiet);
    }

    #[test]
    fn cli_parses_jira_issue_view() {
        let cli =
            Cli::try_parse_from(["shrug", "jira", "issue", "view", "TEAM-123"]).unwrap();
        match cli.command {
            Some(Commands::Jira {
                command: JiraCommands::Issue { .. },
            }) => {}
            _ => panic!("Expected Jira Issue command"),
        }
    }

    #[test]
    fn cli_parses_jira_issue_create() {
        let cli = Cli::try_parse_from([
            "shrug", "jira", "issue", "create",
            "-s", "Test bug",
            "--project", "TEAM",
            "--type", "Bug",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Jira {
                command: JiraCommands::Issue { .. },
            }) => {}
            _ => panic!("Expected Jira Issue Create command"),
        }
    }

    #[test]
    fn cli_parses_profile_create() {
        let cli = Cli::try_parse_from([
            "shrug",
            "profile",
            "create",
            "test",
            "--site",
            "test.atlassian.net",
            "--email",
            "a@b.com",
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
    fn cli_parses_profile_view() {
        let cli = Cli::try_parse_from(["shrug", "profile", "view", "myprofile"]).unwrap();
        match cli.command {
            Some(Commands::Profile {
                command: ProfileCommands::View { ref name },
            }) => assert_eq!(name, "myprofile"),
            _ => panic!("Expected Profile View"),
        }
    }

    #[test]
    fn cli_parses_profile_use() {
        let cli = Cli::try_parse_from(["shrug", "profile", "use", "myprofile"]).unwrap();
        match cli.command {
            Some(Commands::Profile {
                command: ProfileCommands::Use { ref name },
            }) => assert_eq!(name, "myprofile"),
            _ => panic!("Expected Profile Use"),
        }
    }

    #[test]
    fn jql_flags_not_global() {
        let result = Cli::try_parse_from(["shrug", "--project", "KAN"]);
        assert!(result.is_err(), "JQL flags should no longer be global");
    }

    #[test]
    fn jql_flags_parsed_on_static_issue_list() {
        // "issue" (singular) routes to static handler with typed flags
        let cli = Cli::try_parse_from([
            "shrug",
            "jira",
            "issue",
            "list",
            "--project",
            "KAN",
            "--status",
            "Open",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Jira {
                command: JiraCommands::Issue { .. },
            }) => {}
            _ => panic!("Expected Jira Issue List command"),
        }
    }

    #[test]
    fn cli_jira_alias_j() {
        let cli = Cli::try_parse_from(["shrug", "j", "issue", "view", "TEAM-1"]).unwrap();
        match cli.command {
            Some(Commands::Jira {
                command: JiraCommands::Issue { .. },
            }) => {}
            _ => panic!("Expected Jira Issue via alias 'j'"),
        }
    }

    #[test]
    fn cli_jira_software_alias_jsw() {
        let cli = Cli::try_parse_from(["shrug", "jsw", "board", "list"]).unwrap();
        match cli.command {
            Some(Commands::JiraSoftware {
                command: JswCommands::Board { .. },
            }) => {}
            _ => panic!("Expected JiraSoftware Board via alias 'jsw'"),
        }
    }

    #[test]
    fn cli_confluence_alias_c() {
        let cli = Cli::try_parse_from(["shrug", "c", "page", "list"]).unwrap();
        match cli.command {
            Some(Commands::Confluence {
                command: ConfluenceCommands::Page { .. },
            }) => {}
            _ => panic!("Expected Confluence Page via alias 'c'"),
        }
    }

    #[test]
    fn cli_confluence_alias_conf() {
        let cli = Cli::try_parse_from(["shrug", "conf", "page", "view", "123"]).unwrap();
        match cli.command {
            Some(Commands::Confluence {
                command: ConfluenceCommands::Page { .. },
            }) => {}
            _ => panic!("Expected Confluence Page via alias 'conf'"),
        }
    }
}
