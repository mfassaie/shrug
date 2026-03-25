mod handlers;

use std::env;

use clap::Parser;
use owo_colors::OwoColorize;

use shrug::auth::credentials::ResolvedCredential;
use shrug::cli::{Cli, ColorChoice, Commands, JiraCommands, JswCommands, ConfluenceCommands};
use shrug::core::config::{self, ShrugConfig};
use shrug::dynamic_completions;
use shrug::core::error::ShrugError;
use shrug::executor;
use shrug::content::jql;
use shrug::core::logging;
use shrug::spec::registry::Product;

const SIGINT_EXIT: i32 = 130;

fn run(config: &ShrugConfig, cli: &Cli) -> Result<(), ShrugError> {
    match &cli.command {
        Some(Commands::Jira { command }) => {
            let paths = handlers::get_paths()?;
            let profile_store = handlers::get_profile_store(&paths)?;
            let cred_store = handlers::get_credential_store(&paths)?;
            let profile =
                handlers::resolve_profile(&cli.profile, config, &profile_store)?;

            if profile.is_none() && profile_store.list()?.is_empty() {
                return Err(ShrugError::AuthError(
                    "No profile configured. Run `shrug auth setup` to connect your Atlassian account.".into(),
                ));
            }

            let credential = resolve_credential(&cred_store, profile.as_ref());

            match command {
                JiraCommands::Issue { command: issue_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jira::issue::execute(
                        issue_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JiraCommands::Project { command: proj_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jira::project::execute(
                        proj_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JiraCommands::Filter { command: filter_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jira::filter::execute(
                        filter_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JiraCommands::Dashboard { command: dash_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jira::dashboard::execute(
                        dash_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JiraCommands::Label { command: label_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jira::label::execute(
                        label_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JiraCommands::Audit { command: audit_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jira::audit::execute(
                        audit_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JiraCommands::Search { command: search_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jira::search::execute(
                        search_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JiraCommands::External(args) => {
                    let client = executor::create_client()?;

                    // JQL flag extraction for list verb (dynamic entities)
                    let effective_args =
                        if args.len() >= 2 && args[1] == "list" {
                            let (shorthand, raw_jql, cleaned) =
                                jql::extract_jql_flags(&args[2..]);
                            if !shorthand.is_empty() || raw_jql.is_some() {
                                let mut new_args = vec![args[0].clone(), args[1].clone()];
                                if let Some(jql_str) =
                                    shorthand.build_jql(raw_jql.as_deref())
                                {
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
                        Product::Jira,
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
            }
        }
        Some(Commands::JiraSoftware { command }) => {
            let paths = handlers::get_paths()?;
            let profile_store = handlers::get_profile_store(&paths)?;
            let cred_store = handlers::get_credential_store(&paths)?;
            let profile =
                handlers::resolve_profile(&cli.profile, config, &profile_store)?;

            if profile.is_none() && profile_store.list()?.is_empty() {
                return Err(ShrugError::AuthError(
                    "No profile configured. Run `shrug auth setup` to connect your Atlassian account.".into(),
                ));
            }

            let credential = resolve_credential(&cred_store, profile.as_ref());

            match command {
                JswCommands::Board { command: board_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jsw::board::execute(
                        board_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JswCommands::Sprint { command: sprint_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jsw::sprint::execute(
                        sprint_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JswCommands::Epic { command: epic_cmd } => {
                    let cred = credential.ok_or_else(|| {
                        ShrugError::AuthError(
                            "No credentials found. Run `shrug auth setup` to configure.".into(),
                        )
                    })?;
                    let client = shrug::core::http::create_client()?;
                    shrug::jsw::epic::execute(
                        epic_cmd,
                        &cred,
                        &client,
                        &config.output_format,
                        &config.color,
                        cli.limit,
                    )
                }
                JswCommands::External(args) => {
                    let client = executor::create_client()?;

                    // JQL flag extraction for list verb
                    let effective_args =
                        if args.len() >= 2 && args[1] == "list" {
                            let (shorthand, raw_jql, cleaned) =
                                jql::extract_jql_flags(&args[2..]);
                            if !shorthand.is_empty() || raw_jql.is_some() {
                                let mut new_args = vec![args[0].clone(), args[1].clone()];
                                if let Some(jql_str) =
                                    shorthand.build_jql(raw_jql.as_deref())
                                {
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
                        Product::JiraSoftware,
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
            }
        }
        Some(Commands::Confluence { command }) => {
            match command {
                ConfluenceCommands::External(args) => {
                    let paths = handlers::get_paths()?;
                    let profile_store = handlers::get_profile_store(&paths)?;
                    let cred_store = handlers::get_credential_store(&paths)?;
                    let profile =
                        handlers::resolve_profile(&cli.profile, config, &profile_store)?;

                    if profile.is_none() && profile_store.list()?.is_empty() {
                        return Err(ShrugError::AuthError(
                            "No profile configured. Run `shrug auth setup` to connect your Atlassian account.".into(),
                        ));
                    }

                    let credential = resolve_credential(&cred_store, profile.as_ref());
                    let client = executor::create_client()?;

                    handlers::handle_product(
                        Product::Confluence,
                        args,
                        config,
                        &client,
                        credential.as_ref(),
                        cli.dry_run,
                        cli.limit,
                        &config.output_format,
                        &config.color,
                    )
                }
            }
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

    if let Err(e) = cred_store.ensure_fresh_tokens(p) {
        tracing::warn!("Token refresh failed: {}", e);
    }

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

    let trace = cli.verbose >= 3;
    logging::init_logging(cli.verbose, trace, &cli.color);

    if let Err(e) = ctrlc::set_handler(move || {
        eprintln!();
        std::process::exit(SIGINT_EXIT);
    }) {
        tracing::warn!("Failed to set Ctrl+C handler: {e}");
    }

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

    let color_stderr = cli.color != ColorChoice::Never
        && env::var("NO_COLOR").is_err()
        && is_terminal::is_terminal(std::io::stderr());

    match run(&config, &cli) {
        Ok(()) => std::process::exit(shrug::core::exit_codes::OK),
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
