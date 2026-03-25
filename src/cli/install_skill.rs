//! CLI definitions for the install-skill command.

use clap::ValueEnum;

/// Where to install the skill.
#[derive(Clone, Debug, ValueEnum)]
pub enum InstallSkillScope {
    /// Install to ~/.claude/skills/ (available to all projects)
    User,
    /// Install to .claude/skills/ in the current project
    Project,
}
