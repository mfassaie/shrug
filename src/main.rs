use clap::Parser;

use shrug::cli::{Cli, Commands};
use shrug::config::{self, ShrugConfig};
use shrug::error::ShrugError;
use shrug::logging;

const SIGINT_EXIT: i32 = 130;

fn run(_config: &ShrugConfig, cli: &Cli) -> Result<(), ShrugError> {
    match &cli.command {
        Some(Commands::Jira { .. }) => eprintln!("Jira: not yet implemented"),
        Some(Commands::JiraSoftware { .. }) => eprintln!("Jira Software: not yet implemented"),
        Some(Commands::Confluence { .. }) => eprintln!("Confluence: not yet implemented"),
        Some(Commands::Bitbucket { .. }) => eprintln!("Bitbucket: not yet implemented"),
        Some(Commands::Jsm { .. }) => eprintln!("Jira Service Management: not yet implemented"),
        Some(Commands::Auth { .. }) => eprintln!("Auth: not yet implemented"),
        Some(Commands::Profile { .. }) => eprintln!("Profile: not yet implemented"),
        Some(Commands::Cache { .. }) => eprintln!("Cache: not yet implemented"),
        Some(Commands::Completions { .. }) => eprintln!("Completions: not yet implemented"),
        None => {
            eprintln!("Run `shrug --help` for usage information.");
        }
    }
    Ok(())
}

fn main() {
    let _ = enable_ansi_support::enable_ansi_support();

    let cli = Cli::parse();

    // Initialize logging before anything else so config errors are logged
    logging::init_logging(cli.verbose, cli.trace, &cli.color);

    // Set up Ctrl+C handler — non-panicking, falls back to OS default on failure
    if let Err(e) = ctrlc::set_handler(move || {
        eprintln!();
        std::process::exit(SIGINT_EXIT);
    }) {
        tracing::warn!("Failed to set Ctrl+C handler: {e}");
    }

    // Load config with layered precedence, then apply CLI overrides
    let mut config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(e.exit_code());
        }
    };
    config.apply_cli_overrides(&cli.output, &cli.color, &cli.profile);

    tracing::debug!(config = ?config, "Configuration loaded");

    match run(&config, &cli) {
        Ok(()) => std::process::exit(shrug::exit_codes::OK),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(e.exit_code());
        }
    }
}
