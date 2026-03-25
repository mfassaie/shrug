use shrug::auth::credentials::CredentialStore;
use shrug::auth::profile::ProfileStore;
use shrug::cli::profile::ProfileCommands;
use shrug::core::error::ShrugError;

use shrug::auth::profile::Profile;

pub fn handle_profile(
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
                println!("No profiles configured. Create one with: shrug profile create <name> --site <site> --email <email>");
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
        ProfileCommands::Get { name } => {
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
        ProfileCommands::Update {
            name,
            site,
            email,
            auth_type,
        } => {
            let updated =
                store.update(name, site.as_deref(), email.as_deref(), auth_type.as_ref())?;
            println!("Profile '{}' updated.", name);
            println!("Site:      {}", updated.site);
            println!("Email:     {}", updated.email);
            println!("Auth type: {}", updated.auth_type);
        }
        ProfileCommands::Delete { name } => {
            store.delete(name)?;
            // Clean up credentials (non-blocking)
            cred_store.delete(name);
            println!("Profile '{}' deleted.", name);
        }
    }
    Ok(())
}
