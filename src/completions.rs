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

}
