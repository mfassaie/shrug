//! Template command CLI definitions.
//!
//! `shrug template` generates JSON body scaffolds for commands that support --from-json.
//! All output goes to files in --output-dir (never stdout).

use clap::Subcommand;

/// Template generation subcommands.
#[derive(Subcommand)]
pub enum TemplateCommands {
    /// Generate all available JSON templates
    All {
        /// Output directory for generated template files
        #[arg(long)]
        output_dir: String,
    },
    /// Generate Jira JSON templates
    #[command(visible_alias = "j")]
    Jira {
        /// Entity (issue)
        entity: String,
        /// Verb (create, edit)
        verb: String,
        /// Output directory for generated template file
        #[arg(long)]
        output_dir: String,
    },
    /// Generate Jira Software JSON templates
    #[command(name = "jira-software", visible_alias = "jsw")]
    JiraSoftware {
        /// Entity (board, sprint)
        entity: String,
        /// Verb (create, edit)
        verb: String,
        /// Output directory for generated template file
        #[arg(long)]
        output_dir: String,
    },
    /// Generate Confluence JSON templates
    #[command(visible_aliases = ["c", "conf"])]
    Confluence {
        /// Entity (space, page, blogpost, custom-content)
        entity: String,
        /// Verb (create, edit)
        verb: String,
        /// Output directory for generated template file
        #[arg(long)]
        output_dir: String,
    },
}
