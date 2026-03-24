mod handlers;

use std::env;

use clap::Parser;
use owo_colors::OwoColorize;

use shrug::auth::credentials::ResolvedCredential;
use shrug::cli::{Cli, ColorChoice, Commands};
use shrug::config::{self, ShrugConfig};
use shrug::dynamic_completions;
use shrug::error::ShrugError;
use shrug::executor;
use shrug::jql;
use shrug::logging;
use shrug::spec::registry::Product;

const SIGINT_EXIT: i32 = 130;

fn run(config: &ShrugConfig, cli: &Cli) -> Result<(), ShrugError> {
    match &cli.command {
        Some(Commands::Jira { args })
        | Some(Commands::JiraSoftware { args })
        | Some(Commands::Confluence { args }) => {
            let product = match &cli.command {
                Some(Commands::Jira { .. }) => Product::Jira,
                Some(Commands::JiraSoftware { .. }) => Product::JiraSoftware,
                Some(Commands::Confluence { .. }) => Product::Confluence,
                _ => unreachable!(),
            };

            // Resolve active profile and credentials for product commands
            let paths = handlers::get_paths()?;
            let profile_store = handlers::get_profile_store(&paths)?;
            let cred_store = handlers::get_credential_store(&paths)?;
            let profile = handlers::resolve_profile(&cli.profile, config, &profile_store)?;

            // First-run detection: if no profile resolved and profile store is empty,
            // guide the user to set up their account rather than failing with a confusing error.
            if profile.is_none() && profile_store.list()?.is_empty() {
                return Err(ShrugError::AuthError(
                    "No profile configured. Run `shrug auth setup` to connect your Atlassian account.".into(),
                ));
            }

            let credential = resolve_credential(&cred_store, profile.as_ref());

            let client = executor::create_client()?;

            // For Jira/JiraSoftware: extract JQL flags only for `list` verb
            let effective_args = if matches!(product, Product::Jira | Product::JiraSoftware)
                && args.len() >= 2
                && args[1] == "list"
            {
                let (shorthand, raw_jql, cleaned) = jql::extract_jql_flags(&args[2..]);
                if !shorthand.is_empty() || raw_jql.is_some() {
                    let mut new_args = vec![args[0].clone(), args[1].clone()];
                    if let Some(jql_str) = shorthand.build_jql(raw_jql.as_deref()) {
                        new_args.push("--jql".to_string());
                        new_args.push(jql_str);
                    }
                    new_args.extend(cleaned);
                    new_args
                } else {
                    args.clone()
                }
            } else {
                args.clone()
            };

            handlers::handle_product(
                product,
                &effective_args,
                config,
                &client,
                credential.as_ref(),
                cli.dry_run,
                cli.limit,
                &config.output_format,
                &config.color,
            )
        }
        Some(Commands::Auth { command }) => {
            let paths = handlers::get_paths()?;
            let profile_store = handlers::get_profile_store(&paths)?;
            let cred_store = handlers::get_credential_store(&paths)?;
            handlers::handle_auth(command, &profile_store, &cred_store)
        }
        Some(Commands::Profile { command }) => {
            let paths = handlers::get_paths()?;
            let profile_store = handlers::get_profile_store(&paths)?;
            let cred_store = handlers::get_credential_store(&paths)?;
            handlers::handle_profile(command, &profile_store, &cred_store)
        }
        Some(Commands::Cache { command }) => handlers::handle_cache(command, config),
        Some(Commands::Complete {
            completion_type,
            args,
        }) => {
            let paths = handlers::get_paths()?;
            let comp_cache =
                dynamic_completions::CompletionCache::new(paths.cache_dir().to_path_buf())?;
            let profile_store = handlers::get_profile_store(&paths)?;
            let cred_store = handlers::get_credential_store(&paths)?;
            let credential = match handlers::resolve_profile(&cli.profile, config, &profile_store) {
                Ok(Some(ref profile)) => cred_store.resolve(profile, None).ok().flatten(),
                _ => None,
            };
            let comp_client = executor::create_client()?;
            let values = dynamic_completions::complete(
                completion_type,
                &comp_client,
                credential.as_ref(),
                &comp_cache,
                args,
            );
            for value in values {
                println!("{}", value);
            }
            Ok(())
        }
        None => {
            eprintln!("Run `shrug --help` for usage information.");
            Ok(())
        }
    }
}

/// Resolve credentials for the active profile, handling token refresh and fallbacks.
fn resolve_credential(
    cred_store: &shrug::auth::credentials::CredentialStore,
    profile: Option<&shrug::auth::profile::Profile>,
) -> Option<ResolvedCredential> {
    let p = profile?;
    tracing::debug!(profile = %p.name, site = %p.site, "Active profile for request");

    // Ensure OAuth tokens are fresh before resolving
    if let Err(e) = cred_store.ensure_fresh_tokens(p) {
        tracing::warn!("Token refresh failed: {}", e);
    }

    // Resolve credentials (env vars > keychain > encrypted file without password)
    match cred_store.resolve(p, None) {
        Ok(Some(cred)) => {
            tracing::debug!(source = %cred.source, "Credential resolved");
            Some(cred)
        }
        Ok(None) => {
            tracing::debug!("No credentials found for profile");
            None
        }
        Err(e) => {
            tracing::warn!("Credential resolution failed: {}", e);
            None
        }
    }
}

fn main() {
    let _ = enable_ansi_support::enable_ansi_support();

    let cli = Cli::parse();

    // Initialize logging before anything else so config errors are logged
    // -vvv (verbose >= 3) enables trace-level logging (was separate --trace flag)
    let trace = cli.verbose >= 3;
    logging::init_logging(cli.verbose, trace, &cli.color);

    // Set up Ctrl+C handler — non-panicking, falls back to OS default on failure
    if let Err(e) = ctrlc::set_handler(move || {
        eprintln!();
        std::process::exit(SIGINT_EXIT);
    }) {
        tracing::warn!("Failed to set Ctrl+C handler: {e}");
    }

    // Load config with layered precedence, then apply CLI overrides
    // Pre-cli colour check (cli.color not yet available for config errors)
    let color_stderr_early =
        env::var("NO_COLOR").is_err() && is_terminal::is_terminal(std::io::stderr());

    let mut config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            if color_stderr_early {
                eprintln!("{} {e}", "Error:".red());
                eprintln!("{} {}", "Hint:".yellow(), e.remediation());
            } else {
                eprintln!("Error: {e}");
                eprintln!("Hint: {}", e.remediation());
            }
            std::process::exit(e.exit_code());
        }
    };
    config.apply_cli_overrides(&cli.output, &cli.color, &cli.profile);

    tracing::debug!(config = ?config, "Configuration loaded");

    // Post-cli colour check (includes --color flag)
    let color_stderr = cli.color != ColorChoice::Never
        && env::var("NO_COLOR").is_err()
        && is_terminal::is_terminal(std::io::stderr());

    match run(&config, &cli) {
        Ok(()) => std::process::exit(shrug::exit_codes::OK),
        Err(e) => {
            if color_stderr {
                eprintln!("{} {e}", "Error:".red());
                eprintln!("{} {}", "Hint:".yellow(), e.remediation());
            } else {
                eprintln!("Error: {e}");
                eprintln!("Hint: {}", e.remediation());
            }
            std::process::exit(e.exit_code());
        }
    }
}
