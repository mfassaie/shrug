//! Shell completion generator for shrug CLI.
//!
//! Uses clap_complete to generate completion scripts for bash, zsh, fish,
//! and PowerShell from the Cli command definition.

use std::io::Write;

use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::cli::Cli;
use crate::core::error::ShrugError;

const AVAILABLE_SHELLS: &[&str] = &["bash", "zsh", "fish", "powershell"];

/// Generate shell completions and write them to the provided writer.
///
/// Supported shells: bash, zsh, fish, powershell.
/// Returns an error for unknown shell names, listing available options.
pub fn generate_completions(shell: &str, writer: &mut impl Write) -> Result<(), ShrugError> {
    let shell_type = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => {
            return Err(ShrugError::UsageError(format!(
                "Unknown shell '{}'.\n\nAvailable shells:\n{}",
                shell,
                AVAILABLE_SHELLS
                    .iter()
                    .map(|s| format!("  {s}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )));
        }
    };

    let mut cmd = Cli::command();
    generate(shell_type, &mut cmd, "shrug", writer);

    Ok(())
}

/// Generate dynamic completion scripts that call `shrug _complete` for live lookups.
pub fn generate_dynamic_completions(
    shell: &str,
    writer: &mut impl Write,
) -> Result<(), ShrugError> {
    let script = match shell.to_lowercase().as_str() {
        "bash" => BASH_DYNAMIC,
        "zsh" => ZSH_DYNAMIC,
        "fish" => FISH_DYNAMIC,
        "powershell" => POWERSHELL_DYNAMIC,
        _ => {
            return Err(ShrugError::UsageError(format!(
                "Unknown shell '{}'.\n\nAvailable shells:\n{}",
                shell,
                AVAILABLE_SHELLS
                    .iter()
                    .map(|s| format!("  {s}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )));
        }
    };

    writer
        .write_all(script.as_bytes())
        .map_err(|e| ShrugError::SpecError(format!("Failed to write completions: {e}")))?;

    Ok(())
}

const BASH_DYNAMIC: &str = r#"# shrug dynamic completions for bash
# Source this file: eval "$(shrug completions --dynamic bash)"

_shrug_dynamic_complete() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    case "$prev" in
        --project|--projectIdOrKey)
            COMPREPLY=($(compgen -W "$(shrug _complete projects 2>/dev/null)" -- "$cur"))
            return 0
            ;;
        --space)
            COMPREPLY=($(compgen -W "$(shrug _complete spaces 2>/dev/null)" -- "$cur"))
            return 0
            ;;
        --issueIdOrKey)
            COMPREPLY=($(compgen -W "$(shrug _complete issues 2>/dev/null)" -- "$cur"))
            return 0
            ;;
    esac
}

complete -F _shrug_dynamic_complete shrug
"#;

const ZSH_DYNAMIC: &str = r#"# shrug dynamic completions for zsh
# Source this file: eval "$(shrug completions --dynamic zsh)"

_shrug_dynamic() {
    local -a completions

    case "$words[$CURRENT-1]" in
        --project|--projectIdOrKey)
            completions=(${(f)"$(shrug _complete projects 2>/dev/null)"})
            compadd -a completions
            return 0
            ;;
        --space)
            completions=(${(f)"$(shrug _complete spaces 2>/dev/null)"})
            compadd -a completions
            return 0
            ;;
        --issueIdOrKey)
            completions=(${(f)"$(shrug _complete issues 2>/dev/null)"})
            compadd -a completions
            return 0
            ;;
    esac
}

compdef _shrug_dynamic shrug
"#;

const FISH_DYNAMIC: &str = r#"# shrug dynamic completions for fish
# Source this file: shrug completions --dynamic fish | source

complete -c shrug -l project -f -a '(shrug _complete projects 2>/dev/null)'
complete -c shrug -l projectIdOrKey -f -a '(shrug _complete projects 2>/dev/null)'
complete -c shrug -l space -f -a '(shrug _complete spaces 2>/dev/null)'
complete -c shrug -l issueIdOrKey -f -a '(shrug _complete issues 2>/dev/null)'
"#;

const POWERSHELL_DYNAMIC: &str = r#"# shrug dynamic completions for PowerShell
# Source this file: shrug completions --dynamic powershell | Invoke-Expression

Register-ArgumentCompleter -CommandName shrug -ScriptBlock {
    param($commandName, $wordToComplete, $cursorPosition)

    $words = $wordToComplete -split '\s+'
    $prev = if ($words.Count -ge 2) { $words[-2] } else { '' }

    switch ($prev) {
        { $_ -in '--project', '--projectIdOrKey' } {
            (shrug _complete projects 2>$null) -split "`n" | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
        '--space' {
            (shrug _complete spaces 2>$null) -split "`n" | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
        '--issueIdOrKey' {
            (shrug _complete issues 2>$null) -split "`n" | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_completions_produce_output() {
        let mut buf = Vec::new();
        generate_completions("bash", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty(), "Bash completions should produce output");
        assert!(output.contains("shrug"), "Should contain the binary name");
    }

    #[test]
    fn zsh_completions_produce_output() {
        let mut buf = Vec::new();
        generate_completions("zsh", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty(), "Zsh completions should produce output");
    }

    #[test]
    fn fish_completions_produce_output() {
        let mut buf = Vec::new();
        generate_completions("fish", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty(), "Fish completions should produce output");
    }

    #[test]
    fn powershell_completions_produce_output() {
        let mut buf = Vec::new();
        generate_completions("powershell", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(
            !output.is_empty(),
            "PowerShell completions should produce output"
        );
    }

    #[test]
    fn unknown_shell_returns_error() {
        let mut buf = Vec::new();
        let result = generate_completions("unknown", &mut buf);
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown"), "Should name the bad shell");
        assert!(msg.contains("bash"), "Should list available shells");
        assert!(msg.contains("zsh"), "Should list available shells");
    }

    #[test]
    fn shell_name_is_case_insensitive() {
        let mut buf = Vec::new();
        generate_completions("BASH", &mut buf).unwrap();
        assert!(!buf.is_empty());

        let mut buf2 = Vec::new();
        generate_completions("Zsh", &mut buf2).unwrap();
        assert!(!buf2.is_empty());
    }

    #[test]
    fn dynamic_bash_completions_produce_output() {
        let mut buf = Vec::new();
        generate_dynamic_completions("bash", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("_shrug_dynamic_complete"));
        assert!(output.contains("_complete projects"));
    }

    #[test]
    fn dynamic_zsh_completions_produce_output() {
        let mut buf = Vec::new();
        generate_dynamic_completions("zsh", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("_shrug_dynamic"));
        assert!(output.contains("_complete projects"));
    }

    #[test]
    fn dynamic_fish_completions_produce_output() {
        let mut buf = Vec::new();
        generate_dynamic_completions("fish", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("_complete projects"));
        assert!(output.contains("_complete spaces"));
    }

    #[test]
    fn dynamic_powershell_completions_produce_output() {
        let mut buf = Vec::new();
        generate_dynamic_completions("powershell", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Register-ArgumentCompleter"));
        assert!(output.contains("_complete projects"));
    }

    #[test]
    fn dynamic_unknown_shell_returns_error() {
        let mut buf = Vec::new();
        let result = generate_dynamic_completions("unknown", &mut buf);
        assert!(result.is_err());
    }
}
