use std::env;

use clap::Parser;

use shrug::auth::credentials::{CredentialStore, ResolvedCredential};
use shrug::auth::oauth;
use shrug::auth::profile::{AuthType, Profile, ProfileStore};
use shrug::cli::{
    AuthCommands, CacheCommands, Cli, ColorChoice, Commands, OutputFormat, ProfileCommands,
};
use shrug::cmd::router;
use shrug::completions;
use shrug::config::{self, ShrugConfig, ShrugPaths};
use shrug::error::ShrugError;
use shrug::executor;
use shrug::helpers;
use shrug::jql::JqlShorthand;
use shrug::logging;
use shrug::markdown_to_adf;
use shrug::output;
use shrug::spec::registry::Product;
use shrug::spec::SpecCache;
use shrug::spec::SpecLoader;

const SIGINT_EXIT: i32 = 130;

#[allow(clippy::too_many_arguments)]
fn handle_product(
    product: Product,
    args: &[String],
    config: &ShrugConfig,
    client: &reqwest::blocking::Client,
    credential: Option<&ResolvedCredential>,
    dry_run: bool,
    json_body: Option<&str>,
    page_all: bool,
    limit: Option<u32>,
    output_format: &OutputFormat,
    color: &ColorChoice,
    fields: Option<&[String]>,
    no_pager: bool,
) -> Result<(), ShrugError> {
    let paths = ShrugPaths::new()
        .ok_or_else(|| ShrugError::SpecError("Could not determine cache directory".into()))?;
    let cache = SpecCache::new(paths.cache_dir().to_path_buf())?;
    let loader = SpecLoader::new(cache, config.cache_ttl_hours);
    let spec = loader.load(&product)?;

    let resolved = router::route_product(&product, &spec, args)?;

    let parsed_args = executor::parse_args(
        &resolved.operation,
        &resolved.remaining_args,
        json_body.map(|s| s.to_string()),
    )?;

    let is_tty = is_terminal::is_terminal(std::io::stdout());
    let effective_format = output::resolve_format(output_format, is_tty);
    let color_enabled = output::should_use_color(color, is_tty);

    executor::execute(
        client,
        &product,
        &resolved,
        &parsed_args,
        credential,
        dry_run,
        page_all,
        limit,
        &effective_format,
        is_tty,
        color_enabled,
        fields,
        no_pager,
    )
}

/// Resolve the active profile from the precedence chain:
/// --profile flag > SHRUG_PROFILE env > config default_profile > .default file
fn resolve_profile(
    cli_profile: &Option<String>,
    config: &ShrugConfig,
    store: &ProfileStore,
) -> Result<Option<Profile>, ShrugError> {
    if let Some(name) = cli_profile {
        let profile = store.get(name)?;
        tracing::debug!(profile = ?profile.name, "Profile resolved from --profile flag");
        return Ok(Some(profile));
    }

    if let Ok(name) = env::var("SHRUG_PROFILE") {
        if !name.is_empty() {
            let profile = store.get(&name)?;
            tracing::debug!(profile = ?profile.name, "Profile resolved from SHRUG_PROFILE env");
            return Ok(Some(profile));
        }
    }

    if let Some(name) = &config.default_profile {
        let profile = store.get(name)?;
        tracing::debug!(profile = ?profile.name, "Profile resolved from config default_profile");
        return Ok(Some(profile));
    }

    if let Some(profile) = store.get_default()? {
        tracing::debug!(profile = ?profile.name, "Profile resolved from .default file");
        return Ok(Some(profile));
    }

    tracing::debug!("No active profile resolved");
    Ok(None)
}

/// Resolve profile name from explicit arg, or fall back to default.
fn resolve_profile_name(
    explicit: &Option<String>,
    profile_store: &ProfileStore,
) -> Result<String, ShrugError> {
    if let Some(name) = explicit {
        return Ok(name.clone());
    }
    match profile_store.get_default()? {
        Some(p) => Ok(p.name),
        None => Err(ShrugError::UsageError(
            "No profile specified and no default profile set. Use --profile <name> or run: shrug profile use --name <name>".into(),
        )),
    }
}

fn handle_auth(
    command: &AuthCommands,
    profile_store: &ProfileStore,
    cred_store: &CredentialStore,
) -> Result<(), ShrugError> {
    match command {
        AuthCommands::SetToken { profile } => {
            let name = resolve_profile_name(profile, profile_store)?;
            // Verify profile exists
            let _ = profile_store.get(&name)?;

            let token = rpassword::prompt_password("API token: ")
                .map_err(|e| ShrugError::AuthError(format!("Failed to read token: {}", e)))?;

            if token.is_empty() {
                return Err(ShrugError::AuthError("API token cannot be empty".into()));
            }

            // Try keychain first
            if CredentialStore::store_keychain(&name, &token) {
                println!("Token stored for profile '{}' (keychain).", name);
            } else {
                // Keychain unavailable — use encrypted file fallback
                tracing::debug!("Keychain unavailable, falling back to encrypted file");
                eprintln!("Keychain unavailable. Using encrypted file storage.");
                let password =
                    rpassword::prompt_password("Encryption password: ").map_err(|e| {
                        ShrugError::AuthError(format!("Failed to read password: {}", e))
                    })?;
                if password.is_empty() {
                    return Err(ShrugError::AuthError(
                        "Encryption password cannot be empty".into(),
                    ));
                }
                cred_store.store_encrypted(&name, &token, &password)?;
                println!("Token stored for profile '{}' (encrypted file).", name);
            }
        }
        AuthCommands::Status { profile } => {
            let name = resolve_profile_name(profile, profile_store)?;
            let _ = profile_store.get(&name)?;

            match cred_store.credential_source(&name) {
                Some(source) => println!("Profile '{}': token set ({})", name, source),
                None => println!("Profile '{}': token not set", name),
            }
        }
        AuthCommands::Login { profile } => {
            handle_login(profile, profile_store, cred_store)?;
        }
        AuthCommands::Setup => {
            handle_setup(profile_store, cred_store)?;
        }
    }
    Ok(())
}

fn handle_login(
    profile_arg: &Option<String>,
    profile_store: &ProfileStore,
    cred_store: &CredentialStore,
) -> Result<(), ShrugError> {
    let name = resolve_profile_name(profile_arg, profile_store)?;
    let p = profile_store.get(&name)?;

    if p.auth_type != AuthType::OAuth2 {
        return Err(ShrugError::UsageError(
            "This profile uses basic auth. Use `shrug auth set-token` instead.".into(),
        ));
    }

    let config = cred_store.retrieve_oauth_config(&name)?.ok_or_else(|| {
        ShrugError::AuthError(
            "OAuth client credentials not configured. Run `shrug auth setup` first.".into(),
        )
    })?;

    let (url, verifier, state) = oauth::start_auth_flow(&config)?;

    eprintln!("Opening browser for authorization...");
    if let Err(e) = open::that(&url) {
        eprintln!("Failed to open browser: {}", e);
        eprintln!("Open this URL manually:\n{}", url);
    }

    eprintln!(
        "Waiting for callback on http://127.0.0.1:{}/callback...",
        config.redirect_port
    );
    let code = oauth::await_callback(config.redirect_port, &state)?;

    eprintln!("Exchanging authorization code for tokens...");
    let tokens = oauth::exchange_code(&config, &code, &verifier)?;

    cred_store.store_oauth_tokens(&name, &tokens)?;
    println!("OAuth authorization complete for profile '{}'.", name);

    Ok(())
}

fn handle_setup(
    profile_store: &ProfileStore,
    cred_store: &CredentialStore,
) -> Result<(), ShrugError> {
    use dialoguer::{Confirm, Input, Password, Select};

    println!("Welcome to shrug! Let's set up your Atlassian connection.\n");

    // Profile name
    let name: String = Input::new()
        .with_prompt("Profile name")
        .default("default".to_string())
        .interact_text()
        .map_err(|e| ShrugError::AuthError(format!("Input failed: {}", e)))?;

    // Site URL
    let site: String = Input::new()
        .with_prompt("Atlassian site URL (e.g., mysite.atlassian.net)")
        .interact_text()
        .map_err(|e| ShrugError::AuthError(format!("Input failed: {}", e)))?;

    // Email
    let email: String = Input::new()
        .with_prompt("Email address")
        .interact_text()
        .map_err(|e| ShrugError::AuthError(format!("Input failed: {}", e)))?;

    // Auth type
    let auth_options = &["API Token (Basic Auth)", "OAuth 2.0"];
    let auth_selection = Select::new()
        .with_prompt("Authentication type")
        .items(auth_options)
        .default(0)
        .interact()
        .map_err(|e| ShrugError::AuthError(format!("Selection failed: {}", e)))?;

    let auth_type = if auth_selection == 0 {
        AuthType::BasicAuth
    } else {
        AuthType::OAuth2
    };

    // Create profile
    let profile = Profile {
        name: name.clone(),
        site,
        email,
        auth_type: auth_type.clone(),
    };
    let was_first = profile_store.create(&profile)?;
    println!("\nProfile '{}' created.", name);
    if was_first {
        println!("Set as default profile.");
    }

    match auth_type {
        AuthType::BasicAuth => {
            let token = rpassword::prompt_password("API token: ")
                .map_err(|e| ShrugError::AuthError(format!("Failed to read token: {}", e)))?;

            if token.is_empty() {
                eprintln!(
                    "No token provided. Set it later with: shrug auth set-token --profile {}",
                    name
                );
            } else if CredentialStore::store_keychain(&name, &token) {
                println!("Token stored (keychain).");
            } else {
                eprintln!("Keychain unavailable. Using encrypted file storage.");
                let password =
                    rpassword::prompt_password("Encryption password: ").map_err(|e| {
                        ShrugError::AuthError(format!("Failed to read password: {}", e))
                    })?;
                if password.is_empty() {
                    return Err(ShrugError::AuthError(
                        "Encryption password cannot be empty".into(),
                    ));
                }
                cred_store.store_encrypted(&name, &token, &password)?;
                println!("Token stored (encrypted file).");
            }
        }
        AuthType::OAuth2 => {
            let client_id: String = Input::new()
                .with_prompt("OAuth client ID")
                .interact_text()
                .map_err(|e| ShrugError::AuthError(format!("Input failed: {}", e)))?;

            let client_secret: String = Password::new()
                .with_prompt("OAuth client secret")
                .interact()
                .map_err(|e| ShrugError::AuthError(format!("Input failed: {}", e)))?;

            let config = oauth::OAuthConfig {
                client_id,
                client_secret,
                redirect_port: 8456,
            };
            cred_store.store_oauth_config(&name, &config)?;
            println!("OAuth client credentials stored.");

            let launch = Confirm::new()
                .with_prompt("Launch browser to authorize now?")
                .default(true)
                .interact()
                .unwrap_or(false);

            if launch {
                handle_login(&Some(name.clone()), profile_store, cred_store)?;
            } else {
                println!(
                    "Run `shrug auth login --profile {}` to authorize later.",
                    name
                );
            }
        }
    }

    println!(
        "\nProfile '{}' configured! You're ready to use shrug.",
        name
    );
    println!("Try: shrug jira issues list --project YOUR_PROJECT_KEY");

    Ok(())
}

fn handle_profile(
    command: &ProfileCommands,
    store: &ProfileStore,
    cred_store: &CredentialStore,
) -> Result<(), ShrugError> {
    match command {
        ProfileCommands::Create {
            name,
            site,
            email,
            auth_type,
        } => {
            let profile = Profile {
                name: name.clone(),
                site: site.clone(),
                email: email.clone(),
                auth_type: auth_type.clone(),
            };
            let was_first = store.create(&profile)?;
            println!("Profile '{}' created.", name);
            if was_first {
                println!("Set as default profile.");
            }
        }
        ProfileCommands::List => {
            let profiles = store.list()?;
            if profiles.is_empty() {
                println!("No profiles configured. Create one with: shrug profile create --name <name> --site <site> --email <email>");
                return Ok(());
            }
            let header = format!(
                "{:<20} {:<40} {:<30} {}",
                "NAME", "SITE", "EMAIL", "DEFAULT"
            );
            println!("{header}");
            println!("{}", "-".repeat(95));
            for p in &profiles {
                let default_marker = if store.is_default(&p.name) { "*" } else { "" };
                println!(
                    "{:<20} {:<40} {:<30} {}",
                    p.name, p.site, p.email, default_marker
                );
            }
        }
        ProfileCommands::Show { name } => {
            let profile = store.get(name)?;
            let is_default = store.is_default(name);
            let token_status = match cred_store.has_credential(name) {
                Ok(true) => "set",
                _ => "not set",
            };
            println!("Name:      {}", profile.name);
            println!("Site:      {}", profile.site);
            println!("Email:     {}", profile.email);
            println!("Auth type: {}", profile.auth_type);
            println!("Default:   {}", if is_default { "yes" } else { "no" });
            println!("Token:     {}", token_status);
        }
        ProfileCommands::Delete { name } => {
            store.delete(name)?;
            // Clean up credentials (non-blocking)
            cred_store.delete(name);
            println!("Profile '{}' deleted.", name);
        }
        ProfileCommands::Use { name } => {
            store.set_default(name)?;
            println!("Now using profile '{}'.", name);
        }
    }
    Ok(())
}

fn handle_cache(command: &CacheCommands, config: &ShrugConfig) -> Result<(), ShrugError> {
    let paths = ShrugPaths::new()
        .ok_or_else(|| ShrugError::SpecError("Could not determine cache directory".into()))?;
    let cache = SpecCache::new(paths.cache_dir().to_path_buf())?;
    let loader = SpecLoader::new(cache, config.cache_ttl_hours);

    match command {
        CacheCommands::Refresh { product: None } => {
            let results = loader.refresh_all();
            let mut ok_count = 0;
            let mut err_count = 0;
            for (product, result) in &results {
                match result {
                    Ok(spec) => {
                        println!(
                            "  {} — {} operations",
                            product.info().display_name,
                            spec.operations.len()
                        );
                        ok_count += 1;
                    }
                    Err(e) => {
                        eprintln!("  {} — failed: {}", product.info().display_name, e);
                        err_count += 1;
                    }
                }
            }
            println!(
                "\nRefreshed {} specs ({} failed).",
                ok_count, err_count
            );
            if err_count > 0 {
                return Err(ShrugError::SpecError(format!(
                    "{} spec(s) failed to refresh",
                    err_count
                )));
            }
        }
        CacheCommands::Refresh {
            product: Some(name),
        } => {
            let product = Product::from_cli_prefix(name).ok_or_else(|| {
                ShrugError::UsageError(format!(
                    "Unknown product '{}'. Valid products: jira, jira-software, confluence, jsm, bitbucket",
                    name
                ))
            })?;
            let spec = loader.refresh(&product)?;
            println!(
                "{} — {} operations",
                product.info().display_name,
                spec.operations.len()
            );
        }
    }

    Ok(())
}

fn get_paths() -> Result<ShrugPaths, ShrugError> {
    ShrugPaths::new()
        .ok_or_else(|| ShrugError::ProfileError("Could not determine config directory".into()))
}

fn get_profile_store(paths: &ShrugPaths) -> Result<ProfileStore, ShrugError> {
    ProfileStore::new(paths.config_dir().join("profiles"))
}

fn get_credential_store(paths: &ShrugPaths) -> Result<CredentialStore, ShrugError> {
    CredentialStore::new(paths.data_dir().to_path_buf())
}

fn run(config: &ShrugConfig, cli: &Cli) -> Result<(), ShrugError> {
    match &cli.command {
        Some(Commands::Jira { args })
        | Some(Commands::JiraSoftware { args })
        | Some(Commands::Confluence { args })
        | Some(Commands::Bitbucket { args })
        | Some(Commands::Jsm { args }) => {
            let product = match &cli.command {
                Some(Commands::Jira { .. }) => Product::Jira,
                Some(Commands::JiraSoftware { .. }) => Product::JiraSoftware,
                Some(Commands::Confluence { .. }) => Product::Confluence,
                Some(Commands::Bitbucket { .. }) => Product::BitBucket,
                Some(Commands::Jsm { .. }) => Product::JiraServiceManagement,
                _ => unreachable!(),
            };

            // Resolve active profile and credentials for product commands
            let paths = get_paths()?;
            let profile_store = get_profile_store(&paths)?;
            let cred_store = get_credential_store(&paths)?;
            let profile = resolve_profile(&cli.profile, config, &profile_store)?;

            // First-run detection: if no profile resolved and profile store is empty,
            // guide the user to set up their account rather than failing with a confusing error.
            if profile.is_none() && profile_store.list()?.is_empty() {
                return Err(ShrugError::AuthError(
                    "No profile configured. Run `shrug auth setup` to connect your Atlassian account.".into(),
                ));
            }

            let credential = if let Some(ref p) = profile {
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
            } else {
                None
            };

            let client = executor::create_client()?;

            // Convert Markdown fields in --json body to ADF if --markdown is set
            let effective_json = if cli.markdown {
                if let Some(ref body) = cli.json {
                    Some(markdown_to_adf::convert_body_markdown(body)?)
                } else {
                    tracing::warn!("--markdown has no effect without --json");
                    None
                }
            } else {
                cli.json.clone()
            };

            // Build JQL from shorthand flags (Jira/JiraSoftware only)
            let mut effective_args = args.clone();
            let shorthand = JqlShorthand {
                project: cli.project.clone(),
                assignee: cli.assignee.clone(),
                status: cli.status.clone(),
                issue_type: cli.issue_type.clone(),
                priority: cli.priority.clone(),
                label: cli.label.clone(),
            };
            if (!shorthand.is_empty() || cli.jql.is_some())
                && matches!(product, Product::Jira | Product::JiraSoftware)
            {
                if let Some(jql) = shorthand.build_jql(cli.jql.as_deref()) {
                    effective_args.push("--jql".to_string());
                    effective_args.push(jql);
                }
            }

            let parsed_fields: Option<Vec<String>> = cli
                .fields
                .as_ref()
                .map(|f| f.split(',').map(|s| s.trim().to_string()).collect());

            // Intercept helper commands (+create, +search, +transition)
            if helpers::is_helper_command(&effective_args) {
                let helper_name = effective_args[0].trim_start_matches('+');
                let helper_remaining = &effective_args[1..];

                let paths = ShrugPaths::new().ok_or_else(|| {
                    ShrugError::SpecError("Could not determine cache directory".into())
                })?;
                let cache = SpecCache::new(paths.cache_dir().to_path_buf())?;
                let loader = SpecLoader::new(cache, config.cache_ttl_hours);
                let spec = loader.load(&product)?;

                let is_tty = is_terminal::is_terminal(std::io::stdout());
                let effective_format = output::resolve_format(&config.output_format, is_tty);
                let color_enabled = output::should_use_color(&config.color, is_tty);

                return helpers::dispatch_helper(
                    helper_name,
                    &product,
                    helper_remaining,
                    &spec,
                    &client,
                    credential.as_ref(),
                    &shorthand,
                    cli.jql.as_deref(),
                    &effective_format,
                    is_tty,
                    color_enabled,
                    parsed_fields.as_deref(),
                    cli.no_pager,
                    cli.dry_run,
                );
            }

            handle_product(
                product,
                &effective_args,
                config,
                &client,
                credential.as_ref(),
                cli.dry_run,
                effective_json.as_deref(),
                cli.page_all,
                cli.limit,
                &config.output_format,
                &config.color,
                parsed_fields.as_deref(),
                cli.no_pager,
            )
        }
        Some(Commands::Auth { command }) => {
            let paths = get_paths()?;
            let profile_store = get_profile_store(&paths)?;
            let cred_store = get_credential_store(&paths)?;
            handle_auth(command, &profile_store, &cred_store)
        }
        Some(Commands::Profile { command }) => {
            let paths = get_paths()?;
            let profile_store = get_profile_store(&paths)?;
            let cred_store = get_credential_store(&paths)?;
            handle_profile(command, &profile_store, &cred_store)
        }
        Some(Commands::Cache { command }) => handle_cache(command, config),
        Some(Commands::Completions { shell }) => {
            completions::generate_completions(shell, &mut std::io::stdout())
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
            eprintln!("Hint: {}", e.remediation());
            std::process::exit(e.exit_code());
        }
    };
    config.apply_cli_overrides(&cli.output, &cli.color, &cli.profile);

    tracing::debug!(config = ?config, "Configuration loaded");

    match run(&config, &cli) {
        Ok(()) => std::process::exit(shrug::exit_codes::OK),
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("Hint: {}", e.remediation());
            std::process::exit(e.exit_code());
        }
    }
}
