use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::error::ShrugError;

/// Authentication type for a profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    /// Email + API token (Basic Auth)
    #[default]
    BasicAuth,
    /// OAuth 2.0 three-legged flow
    OAuth2,
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthType::BasicAuth => write!(f, "basic-auth"),
            AuthType::OAuth2 => write!(f, "oauth2"),
        }
    }
}

/// A named profile containing connection metadata for an Atlassian Cloud site.
/// Credentials are NOT stored here — see Plan 04-02 for keychain/encrypted storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub site: String,
    pub email: String,
    #[serde(default)]
    pub auth_type: AuthType,
}

/// Manages profile TOML files on disk.
///
/// Storage layout:
/// ```text
/// {profiles_dir}/
///   work.toml          # profile file
///   personal.toml      # profile file
///   .default           # plain-text file containing default profile name
/// ```
pub struct ProfileStore {
    profiles_dir: PathBuf,
}

impl ProfileStore {
    /// Create a new ProfileStore, creating the profiles directory if it doesn't exist.
    pub fn new(profiles_dir: PathBuf) -> Result<Self, ShrugError> {
        if !profiles_dir.exists() {
            fs::create_dir_all(&profiles_dir).map_err(|e| {
                ShrugError::ProfileError(format!(
                    "Failed to create profiles directory {}: {}",
                    profiles_dir.display(),
                    e
                ))
            })?;
        }
        Ok(Self { profiles_dir })
    }

    /// Create a new profile. If this is the first profile, it is automatically set as default.
    pub fn create(&self, profile: &Profile) -> Result<bool, ShrugError> {
        Self::validate_name(&profile.name)?;
        Self::validate_site(&profile.site)?;
        Self::validate_email(&profile.email)?;

        let path = self.profile_path(&profile.name);
        if path.exists() {
            return Err(ShrugError::ProfileError(format!(
                "Profile '{}' already exists",
                profile.name
            )));
        }

        // Normalize site before saving
        let mut normalized = profile.clone();
        normalized.site = Self::normalize_site(&profile.site);

        self.write_profile_atomic(&normalized)?;

        // Auto-default if first profile
        let is_first = self.get_default()?.is_none();
        if is_first {
            self.write_default_file(&profile.name)?;
        }

        Ok(is_first)
    }

    /// List all profiles. Skips corrupted files with a warning.
    pub fn list(&self) -> Result<Vec<Profile>, ShrugError> {
        let mut profiles = Vec::new();

        let entries = fs::read_dir(&self.profiles_dir).map_err(|e| {
            ShrugError::ProfileError(format!(
                "Failed to read profiles directory {}: {}",
                self.profiles_dir.display(),
                e
            ))
        })?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Failed to read directory entry in profiles: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            match self.load_profile_file(&path) {
                Ok(profile) => profiles.push(profile),
                Err(e) => {
                    tracing::warn!("Skipping corrupted profile file {}: {}", path.display(), e);
                }
            }
        }

        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(profiles)
    }

    /// Get a profile by name.
    pub fn get(&self, name: &str) -> Result<Profile, ShrugError> {
        let path = self.profile_path(name);
        if !path.exists() {
            return Err(ShrugError::NotFound(format!(
                "Profile '{}' not found",
                name
            )));
        }
        self.load_profile_file(&path)
    }

    /// Delete a profile by name. Also clears .default if this was the default profile.
    pub fn delete(&self, name: &str) -> Result<(), ShrugError> {
        let path = self.profile_path(name);
        if !path.exists() {
            return Err(ShrugError::NotFound(format!(
                "Profile '{}' not found",
                name
            )));
        }

        fs::remove_file(&path).map_err(|e| {
            ShrugError::ProfileError(format!("Failed to delete profile '{}': {}", name, e))
        })?;

        // Clear default if this was the default profile
        if let Ok(Some(default_name)) = self.read_default_name() {
            if default_name == name {
                let _ = fs::remove_file(self.default_path());
            }
        }

        Ok(())
    }

    /// Set a profile as the default. The profile must exist.
    pub fn set_default(&self, name: &str) -> Result<(), ShrugError> {
        // Verify profile exists
        let path = self.profile_path(name);
        if !path.exists() {
            return Err(ShrugError::NotFound(format!(
                "Profile '{}' not found",
                name
            )));
        }

        self.write_default_file(name)
    }

    /// Get the current default profile, if any.
    pub fn get_default(&self) -> Result<Option<Profile>, ShrugError> {
        match self.read_default_name()? {
            Some(name) => {
                let path = self.profile_path(&name);
                if path.exists() {
                    Ok(Some(self.load_profile_file(&path)?))
                } else {
                    // .default points to a deleted profile — clean up
                    let _ = fs::remove_file(self.default_path());
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Update an existing profile. Only non-None fields are changed.
    pub fn update(
        &self,
        name: &str,
        site: Option<&str>,
        email: Option<&str>,
        auth_type: Option<&AuthType>,
    ) -> Result<Profile, ShrugError> {
        let mut profile = self.get(name)?;

        if let Some(new_site) = site {
            Self::validate_site(new_site)?;
            profile.site = Self::normalize_site(new_site);
        }
        if let Some(new_email) = email {
            Self::validate_email(new_email)?;
            profile.email = new_email.to_string();
        }
        if let Some(new_auth_type) = auth_type {
            profile.auth_type = new_auth_type.clone();
        }

        self.write_profile_atomic(&profile)?;
        Ok(profile)
    }

    /// Check if a given profile name is the current default.
    pub fn is_default(&self, name: &str) -> bool {
        self.read_default_name()
            .ok()
            .flatten()
            .map(|d| d == name)
            .unwrap_or(false)
    }

    // --- Private helpers ---

    fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.toml", name))
    }

    fn default_path(&self) -> PathBuf {
        self.profiles_dir.join(".default")
    }

    fn write_default_file(&self, name: &str) -> Result<(), ShrugError> {
        let path = self.default_path();
        let tmp = self.profiles_dir.join(".default.tmp");
        fs::write(&tmp, name).map_err(|e| {
            ShrugError::ProfileError(format!("Failed to write default file: {}", e))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            ShrugError::ProfileError(format!("Failed to set default profile: {}", e))
        })?;
        Ok(())
    }

    fn read_default_name(&self) -> Result<Option<String>, ShrugError> {
        let path = self.default_path();
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| ShrugError::ProfileError(format!("Failed to read default file: {}", e)))?;
        let name = content.trim().to_string();
        if name.is_empty() {
            Ok(None)
        } else {
            Ok(Some(name))
        }
    }

    fn write_profile_atomic(&self, profile: &Profile) -> Result<(), ShrugError> {
        let content = toml::to_string_pretty(profile).map_err(|e| {
            ShrugError::ProfileError(format!(
                "Failed to serialize profile '{}': {}",
                profile.name, e
            ))
        })?;

        let path = self.profile_path(&profile.name);
        let tmp = self.profiles_dir.join(format!("{}.toml.tmp", profile.name));

        fs::write(&tmp, &content).map_err(|e| {
            ShrugError::ProfileError(format!("Failed to write profile '{}': {}", profile.name, e))
        })?;

        fs::rename(&tmp, &path).map_err(|e| {
            // Clean up temp file on rename failure
            let _ = fs::remove_file(&tmp);
            ShrugError::ProfileError(format!("Failed to save profile '{}': {}", profile.name, e))
        })?;

        Ok(())
    }

    fn load_profile_file(&self, path: &Path) -> Result<Profile, ShrugError> {
        let content = fs::read_to_string(path).map_err(|e| {
            ShrugError::ProfileError(format!(
                "Failed to read profile file {}: {}",
                path.display(),
                e
            ))
        })?;
        let profile: Profile = toml::from_str(&content).map_err(|e| {
            ShrugError::ProfileError(format!("Invalid profile file {}: {}", path.display(), e))
        })?;
        Ok(profile)
    }

    fn validate_name(name: &str) -> Result<(), ShrugError> {
        if name.is_empty() || name.len() > 64 {
            return Err(ShrugError::ProfileError(
                "Profile name must be 1-64 characters".into(),
            ));
        }

        let mut chars = name.chars();

        // First character must be alphanumeric lowercase
        match chars.next() {
            Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
            _ => {
                return Err(ShrugError::ProfileError(
                    "Profile name must start with a lowercase letter or digit".into(),
                ));
            }
        }

        // Remaining characters: lowercase alphanumeric or hyphen
        for c in chars {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                return Err(ShrugError::ProfileError(format!(
                    "Profile name contains invalid character '{}'. Only lowercase letters, digits, and hyphens are allowed",
                    c
                )));
            }
        }

        Ok(())
    }

    fn validate_site(site: &str) -> Result<(), ShrugError> {
        let normalized = Self::normalize_site(site);
        // After normalization (which adds https://), strip the scheme and check for a dot
        let host = normalized.strip_prefix("https://").unwrap_or(&normalized);
        if !host.contains('.') {
            return Err(ShrugError::ProfileError(
                "Site URL must contain a valid domain (e.g., mysite.atlassian.net)".into(),
            ));
        }
        if host.is_empty() {
            return Err(ShrugError::ProfileError(
                "Site URL must not be empty".into(),
            ));
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<(), ShrugError> {
        if !email.contains('@') {
            return Err(ShrugError::ProfileError(
                "Email must contain '@' (e.g., user@example.com)".into(),
            ));
        }
        Ok(())
    }

    fn normalize_site(site: &str) -> String {
        let mut s = site.trim().to_string();

        // Strip any existing scheme and re-add https://
        if let Some(stripped) = s.strip_prefix("https://") {
            s = stripped.to_string();
        } else if let Some(stripped) = s.strip_prefix("http://") {
            s = stripped.to_string();
        }

        // Strip trailing slash
        while s.ends_with('/') {
            s.pop();
        }

        format!("https://{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store(dir: &tempfile::TempDir) -> ProfileStore {
        ProfileStore::new(dir.path().join("profiles")).unwrap()
    }

    fn make_profile(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            site: "test.atlassian.net".to_string(),
            email: "user@example.com".to_string(),
            auth_type: AuthType::default(),
        }
    }

    #[test]
    fn create_profile_writes_valid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let profile = make_profile("work");

        store.create(&profile).unwrap();

        let path = dir.path().join("profiles").join("work.toml");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        let loaded: Profile = toml::from_str(&content).unwrap();
        assert_eq!(loaded.name, "work");
        assert_eq!(loaded.site, "https://test.atlassian.net");
        assert_eq!(loaded.email, "user@example.com");
    }

    #[test]
    fn list_returns_all_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        store.create(&make_profile("alpha")).unwrap();
        store.create(&make_profile("beta")).unwrap();

        let profiles = store.list().unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].name, "alpha");
        assert_eq!(profiles[1].name, "beta");
    }

    #[test]
    fn get_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();

        let profile = store.get("work").unwrap();
        assert_eq!(profile.name, "work");
        assert_eq!(profile.site, "https://test.atlassian.net");
    }

    #[test]
    fn get_nonexistent_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        let err = store.get("nonexistent").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("not found"), "Expected not found: {msg}");
        assert_eq!(err.exit_code(), crate::exit_codes::ERROR);
    }

    #[test]
    fn delete_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();

        store.delete("work").unwrap();

        let path = dir.path().join("profiles").join("work.toml");
        assert!(!path.exists());
    }

    #[test]
    fn delete_nonexistent_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        let err = store.delete("nonexistent").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("not found"), "Expected not found: {msg}");
        assert_eq!(err.exit_code(), crate::exit_codes::ERROR);
    }

    #[test]
    fn set_default_writes_default_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();
        store.create(&make_profile("personal")).unwrap();

        store.set_default("personal").unwrap();

        let content = fs::read_to_string(dir.path().join("profiles").join(".default")).unwrap();
        assert_eq!(content.trim(), "personal");
    }

    #[test]
    fn set_default_nonexistent_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        let err = store.set_default("nonexistent").unwrap_err();
        assert_eq!(err.exit_code(), crate::exit_codes::ERROR);
    }

    #[test]
    fn get_default_reads_from_default_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();
        store.create(&make_profile("personal")).unwrap();
        store.set_default("personal").unwrap();

        let default = store.get_default().unwrap().unwrap();
        assert_eq!(default.name, "personal");
    }

    #[test]
    fn first_profile_auto_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        let was_first = store.create(&make_profile("work")).unwrap();
        assert!(was_first);

        let default = store.get_default().unwrap().unwrap();
        assert_eq!(default.name, "work");

        // Second profile should NOT auto-default
        let was_first = store.create(&make_profile("personal")).unwrap();
        assert!(!was_first);

        let default = store.get_default().unwrap().unwrap();
        assert_eq!(default.name, "work"); // Still work
    }

    #[test]
    fn delete_default_profile_clears_default_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();

        assert!(store.get_default().unwrap().is_some());

        store.delete("work").unwrap();

        assert!(store.get_default().unwrap().is_none());
        assert!(!dir.path().join("profiles").join(".default").exists());
    }

    #[test]
    fn invalid_name_spaces() {
        let err = ProfileStore::validate_name("has space").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("invalid character"), "{msg}");
    }

    #[test]
    fn invalid_name_uppercase() {
        let err = ProfileStore::validate_name("Work").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("lowercase"), "{msg}");
    }

    #[test]
    fn invalid_name_too_long() {
        let long_name = "a".repeat(65);
        let err = ProfileStore::validate_name(&long_name).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("1-64"), "{msg}");
    }

    #[test]
    fn invalid_name_empty() {
        let err = ProfileStore::validate_name("").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("1-64"), "{msg}");
    }

    #[test]
    fn valid_name_with_hyphens_and_digits() {
        assert!(ProfileStore::validate_name("my-profile-1").is_ok());
        assert!(ProfileStore::validate_name("a").is_ok());
        assert!(ProfileStore::validate_name("1-prod").is_ok());
    }

    #[test]
    fn invalid_site_no_dots() {
        let err = ProfileStore::validate_site("localhost").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("valid domain"), "{msg}");
    }

    #[test]
    fn invalid_email_no_at() {
        let err = ProfileStore::validate_email("notanemail").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("@"), "{msg}");
    }

    #[test]
    fn duplicate_name_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();

        let err = store.create(&make_profile("work")).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("already exists"), "{msg}");
    }

    #[test]
    fn normalize_site_adds_https_prefix() {
        assert_eq!(
            ProfileStore::normalize_site("mysite.atlassian.net"),
            "https://mysite.atlassian.net"
        );
    }

    #[test]
    fn normalize_site_strips_trailing_slash() {
        assert_eq!(
            ProfileStore::normalize_site("https://mysite.atlassian.net/"),
            "https://mysite.atlassian.net"
        );
    }

    #[test]
    fn normalize_site_replaces_http_with_https() {
        assert_eq!(
            ProfileStore::normalize_site("http://mysite.atlassian.net"),
            "https://mysite.atlassian.net"
        );
    }

    #[test]
    fn normalize_site_preserves_existing_https() {
        assert_eq!(
            ProfileStore::normalize_site("https://mysite.atlassian.net"),
            "https://mysite.atlassian.net"
        );
    }

    #[test]
    fn list_skips_corrupted_toml_files() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("good")).unwrap();

        // Write a corrupted TOML file
        let bad_path = dir.path().join("profiles").join("bad.toml");
        fs::write(&bad_path, "this is not valid toml {{{{").unwrap();

        let profiles = store.list().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "good");
    }

    #[test]
    fn is_default_returns_correct_value() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();
        store.create(&make_profile("personal")).unwrap();

        assert!(store.is_default("work")); // auto-defaulted
        assert!(!store.is_default("personal"));

        store.set_default("personal").unwrap();
        assert!(!store.is_default("work"));
        assert!(store.is_default("personal"));
    }

    #[test]
    fn auth_type_display() {
        assert_eq!(format!("{}", AuthType::BasicAuth), "basic-auth");
        assert_eq!(format!("{}", AuthType::OAuth2), "oauth2");
    }

    #[test]
    fn auth_type_default_is_basic_auth() {
        assert_eq!(AuthType::default(), AuthType::BasicAuth);
    }

    #[test]
    fn update_partial_changes_only_specified_fields() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();

        let updated = store
            .update("work", Some("new-site.atlassian.net"), None, None)
            .unwrap();
        assert_eq!(updated.site, "https://new-site.atlassian.net");
        assert_eq!(updated.email, "user@example.com"); // unchanged
        assert_eq!(updated.auth_type, AuthType::BasicAuth); // unchanged
    }

    #[test]
    fn update_all_fields() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        store.create(&make_profile("work")).unwrap();

        let updated = store
            .update(
                "work",
                Some("other.atlassian.net"),
                Some("new@example.com"),
                Some(&AuthType::OAuth2),
            )
            .unwrap();
        assert_eq!(updated.site, "https://other.atlassian.net");
        assert_eq!(updated.email, "new@example.com");
        assert_eq!(updated.auth_type, AuthType::OAuth2);
    }

    #[test]
    fn update_nonexistent_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        let result = store.update("ghost", Some("x.atlassian.net"), None, None);
        assert!(result.is_err());
    }
}
