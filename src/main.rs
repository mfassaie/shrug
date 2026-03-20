use clap::Parser;

use shrug::cli::{Cli, Commands};
use shrug::cmd::{router, tree};
use shrug::config::{self, ShrugConfig, ShrugPaths};
use shrug::error::ShrugError;
use shrug::logging;
use shrug::spec::registry::Product;
use shrug::spec::SpecCache;
use shrug::spec::SpecLoader;

const SIGINT_EXIT: i32 = 130;

fn handle_product(
    product: Product,
    args: &[String],
    config: &ShrugConfig,
) -> Result<(), ShrugError> {
    let paths = ShrugPaths::new()
        .ok_or_else(|| ShrugError::SpecError("Could not determine cache directory".into()))?;
    let cache = SpecCache::new(paths.cache_dir().to_path_buf())?;
    let loader = SpecLoader::new(cache, config.cache_ttl_hours);
    let spec = loader.load(&product)?;

    let resolved = router::route_product(&product, &spec, args)?;

    // Display operation detail (actual HTTP execution in Phase 5)
    eprintln!(
        "{}",
        tree::format_operation_detail(&resolved.operation, resolved.server_url.as_deref())
    );
    if !resolved.remaining_args.is_empty() {
        eprintln!("\n  Args: {:?}", resolved.remaining_args);
    }
    eprintln!("\n  [Phase 5 will execute this API call]");

    Ok(())
}

fn run(config: &ShrugConfig, cli: &Cli) -> Result<(), ShrugError> {
    match &cli.command {
        Some(Commands::Jira { args }) => handle_product(Product::Jira, args, config),
        Some(Commands::JiraSoftware { args }) => {
            handle_product(Product::JiraSoftware, args, config)
        }
        Some(Commands::Confluence { args }) => handle_product(Product::Confluence, args, config),
        Some(Commands::Bitbucket { args }) => handle_product(Product::BitBucket, args, config),
        Some(Commands::Jsm { args }) => {
            handle_product(Product::JiraServiceManagement, args, config)
        }
        Some(Commands::Auth { .. }) => {
            eprintln!("Auth: not yet implemented");
            Ok(())
        }
        Some(Commands::Profile { .. }) => {
            eprintln!("Profile: not yet implemented");
            Ok(())
        }
        Some(Commands::Cache { .. }) => {
            eprintln!("Cache: not yet implemented");
            Ok(())
        }
        Some(Commands::Completions { .. }) => {
            eprintln!("Completions: not yet implemented");
            Ok(())
        }
        None => {
            eprintln!("Run `shrug --help` for usage information.");
            Ok(())
        }
    }
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
