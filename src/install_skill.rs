//! Install the embedded jira-confluence-cli skill to the user's Claude environment.

use std::fs;
use std::path::PathBuf;

use include_dir::{include_dir, Dir};

use crate::cli::install_skill::InstallSkillScope;
use crate::core::error::ShrugError;

/// The skill directory, embedded at compile time.
static SKILL_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/skills/jira-confluence-cli");

/// Execute the install-skill command.
pub fn execute(scope: &InstallSkillScope) -> Result<(), ShrugError> {
    let target = resolve_target(scope)?;
    let count = extract_all(&SKILL_DIR, &target)?;

    let existed = count.updated > 0;
    eprintln!(
        "Installed {} file{} to {}{}",
        count.total,
        if count.total == 1 { "" } else { "s" },
        target.display(),
        if existed { " (updated)" } else { "" },
    );

    Ok(())
}

/// Resolve the target directory based on scope.
fn resolve_target(scope: &InstallSkillScope) -> Result<PathBuf, ShrugError> {
    match scope {
        InstallSkillScope::User => {
            let home = dirs_home().ok_or_else(|| {
                ShrugError::UsageError("Could not determine home directory.".into())
            })?;
            Ok(home.join(".claude").join("skills").join("jira-confluence-cli"))
        }
        InstallSkillScope::Project => {
            let cwd = std::env::current_dir().map_err(|e| {
                ShrugError::UsageError(format!("Could not read current directory: {}", e))
            })?;
            let claude_dir = cwd.join(".claude");
            if !claude_dir.is_dir() {
                return Err(ShrugError::UsageError(
                    "No .claude directory found in the current directory. \
                     Run this from a Claude Code project root."
                        .into(),
                ));
            }
            Ok(claude_dir.join("skills").join("jira-confluence-cli"))
        }
    }
}

struct WriteCount {
    total: usize,
    updated: usize,
}

/// Extract all embedded files to the filesystem, preserving directory structure.
///
/// Uses `Dir::files()` at the root level only. Each file's `path()` is relative
/// to the include root (e.g. `SKILL.md`, `references/command-reference.md`),
/// so we join directly with target. No recursion needed.
fn extract_all(dir: &Dir, target: &PathBuf) -> Result<WriteCount, ShrugError> {
    fs::create_dir_all(target).map_err(|e| {
        ShrugError::UsageError(format!(
            "Failed to create directory '{}': {}",
            target.display(),
            e
        ))
    })?;

    let mut count = WriteCount {
        total: 0,
        updated: 0,
    };

    // Collect all files from the entire tree (include_dir provides a flat iterator
    // where each file's path() is relative to the root).
    for entry in all_files(dir) {
        let dest = target.join(entry.path());
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ShrugError::UsageError(format!(
                    "Failed to create directory '{}': {}",
                    parent.display(),
                    e
                ))
            })?;
        }
        let existed = dest.exists();
        fs::write(&dest, entry.contents()).map_err(|e| {
            ShrugError::UsageError(format!("Failed to write '{}': {}", dest.display(), e))
        })?;
        count.total += 1;
        if existed {
            count.updated += 1;
        }
    }

    Ok(count)
}

/// Collect all files from a Dir and its subdirectories.
fn all_files<'a>(dir: &'a Dir<'a>) -> Vec<&'a include_dir::File<'a>> {
    let mut files: Vec<&'a include_dir::File<'a>> = dir.files().collect();
    for subdir in dir.dirs() {
        files.extend(all_files(subdir));
    }
    files
}

/// Get the user's home directory.
fn dirs_home() -> Option<PathBuf> {
    directories::UserDirs::new().map(|d| d.home_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_dir_is_not_empty() {
        assert!(
            !SKILL_DIR.files().collect::<Vec<_>>().is_empty()
                || !SKILL_DIR.dirs().collect::<Vec<_>>().is_empty(),
            "Embedded skill directory should contain files or subdirectories"
        );
    }

    #[test]
    fn skill_dir_contains_skill_md() {
        assert!(
            SKILL_DIR.get_file("SKILL.md").is_some(),
            "Embedded skill directory should contain SKILL.md"
        );
    }

    #[test]
    fn user_scope_resolves_to_home() {
        let target = resolve_target(&InstallSkillScope::User).unwrap();
        let path_str = target.to_string_lossy();
        assert!(
            path_str.contains(".claude")
                && path_str.contains("skills")
                && path_str.contains("jira-confluence-cli"),
            "User scope should resolve under ~/.claude/skills/jira-confluence-cli, got: {}",
            path_str
        );
    }
}
