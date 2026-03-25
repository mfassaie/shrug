mod auth;
mod cache;
mod product;
mod profile;

pub use auth::handle_auth;
pub use cache::handle_cache;
pub use product::handle_product;
pub use profile::handle_profile;

use std::env;

use shrug::auth::credentials::CredentialStore;
use shrug::auth::profile::{Profile, ProfileStore};
use shrug::core::config::{ShrugConfig, ShrugPaths};
use shrug::core::error::ShrugError;

/// Resolve the active profile from the precedence chain:
/// --profile flag > SHRUG_PROFILE env > config default_profile > .default file
pub fn resolve_profile(
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

pub fn get_paths() -> Result<ShrugPaths, ShrugError> {
    ShrugPaths::new()
        .ok_or_else(|| ShrugError::ProfileError("Could not determine config directory".into()))
}

pub fn get_profile_store(paths: &ShrugPaths) -> Result<ProfileStore, ShrugError> {
    ProfileStore::new(paths.config_dir().join("profiles"))
}

pub fn get_credential_store(paths: &ShrugPaths) -> Result<CredentialStore, ShrugError> {
    CredentialStore::new(paths.data_dir().to_path_buf())
}
