use shrug::auth::credentials::CredentialStore;
use shrug::auth::oauth;
use shrug::auth::profile::{AuthType, Profile, ProfileStore};
use shrug::cli::auth::AuthCommands;
use shrug::core::error::ShrugError;

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
            "No profile specified and no default profile set. Use --profile <name> or create a profile named 'default'.".into(),
        )),
    }
}

pub fn handle_auth(
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

            // Try keychain first, then token file fallback
            if CredentialStore::store_keychain(&name, &token) {
                println!("Token stored for profile '{}' (keychain).", name);
            } else {
                // Keychain unavailable — use token file fallback (chmod 600)
                tracing::debug!("Keychain unavailable, falling back to token file");
                cred_store.store_token_file(&name, &token)?;
                println!("Token stored for profile '{}' (token-file).", name);
                eprintln!(
                    "Note: Stored in a permission-restricted file. \
                     For stronger security, configure an OS keychain."
                );
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
                cred_store.store_token_file(&name, &token)?;
                println!("Token stored (token-file).");
                eprintln!(
                    "Note: Stored in a permission-restricted file. \
                     For stronger security, configure an OS keychain."
                );
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
