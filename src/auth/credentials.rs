use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::auth::oauth::{self, OAuthConfig, OAuthTokens};
use crate::auth::profile::{AuthType, Profile};
use crate::error::ShrugError;

/// Source of a resolved credential.
#[derive(Debug, Clone, PartialEq)]
pub enum CredentialSource {
    /// From OS keychain (Windows Credential Manager, macOS Keychain, Linux Secret Service)
    Keychain,
    /// From encrypted file fallback
    EncryptedFile,
    /// From permission-restricted token file (chmod 600)
    TokenFile,
    /// From environment variables
    Environment,
}

impl std::fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialSource::Keychain => write!(f, "keychain"),
            CredentialSource::EncryptedFile => write!(f, "encrypted-file"),
            CredentialSource::TokenFile => write!(f, "token-file"),
            CredentialSource::Environment => write!(f, "environment"),
        }
    }
}

/// Authentication scheme for HTTP requests.
#[derive(Debug, Clone)]
pub enum AuthScheme {
    /// Basic Auth: email + API token
    Basic { email: String, api_token: String },
    /// Bearer token from OAuth 2.0
    Bearer { access_token: String },
}

/// A fully resolved credential ready for use in HTTP requests.
#[derive(Debug, Clone)]
pub struct ResolvedCredential {
    pub site: String,
    pub source: CredentialSource,
    pub scheme: AuthScheme,
}

/// Manages credential storage with keychain primary and encrypted file fallback.
///
/// All interactive I/O (password prompts) happens at the CLI layer.
/// This struct receives passwords as parameters — never prompts directly.
pub struct CredentialStore {
    credentials_dir: PathBuf,
}

impl CredentialStore {
    /// Create a new CredentialStore, creating the credentials directory if needed.
    pub fn new(data_dir: PathBuf) -> Result<Self, ShrugError> {
        let credentials_dir = data_dir.join("credentials");
        if !credentials_dir.exists() {
            fs::create_dir_all(&credentials_dir).map_err(|e| {
                ShrugError::AuthError(format!(
                    "Failed to create credentials directory {}: {}",
                    credentials_dir.display(),
                    e
                ))
            })?;
        }
        Ok(Self { credentials_dir })
    }

    // --- API Token Storage (Basic Auth) ---

    /// Store token in OS keychain. Returns true if stored and verified, false if
    /// keychain unavailable or the write did not persist (common on Linux when
    /// the Secret Service daemon is not running or the keyring is locked).
    pub fn store_keychain(profile_name: &str, token: &str) -> bool {
        let Ok(entry) = keyring::Entry::new("shrug", profile_name) else {
            return false;
        };
        if entry.set_password(token).is_err() {
            return false;
        }
        // Verify the write persisted by reading it back
        Self::retrieve_keychain(profile_name).is_some()
    }

    /// Store token encrypted with a password (fallback when keychain unavailable).
    pub fn store_encrypted(
        &self,
        profile_name: &str,
        token: &str,
        password: &str,
    ) -> Result<(), ShrugError> {
        let blob = encrypt_token(token, password)?;
        let json = serde_json::to_string_pretty(&blob).map_err(|e| {
            ShrugError::AuthError(format!("Failed to serialize encrypted credential: {}", e))
        })?;

        let path = self.enc_file_path(profile_name);
        let tmp = self
            .credentials_dir
            .join(format!("{}.enc.tmp", profile_name));

        fs::write(&tmp, &json).map_err(|e| {
            ShrugError::AuthError(format!("Failed to write credential file: {}", e))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            ShrugError::AuthError(format!("Failed to save credential file: {}", e))
        })?;

        Ok(())
    }

    /// Store token in a permission-restricted file (chmod 600).
    /// Fallback when keychain is unavailable and the user doesn't want
    /// an encryption password. Less secure than keychain but works everywhere.
    pub fn store_token_file(&self, profile_name: &str, token: &str) -> Result<(), ShrugError> {
        let path = self.token_file_path(profile_name);
        let tmp = self
            .credentials_dir
            .join(format!("{}.token.tmp", profile_name));

        fs::write(&tmp, token)
            .map_err(|e| ShrugError::AuthError(format!("Failed to write token file: {}", e)))?;

        // Set file permissions to owner-only (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&tmp, perms).map_err(|e| {
                let _ = fs::remove_file(&tmp);
                ShrugError::AuthError(format!("Failed to set token file permissions: {}", e))
            })?;
        }

        fs::rename(&tmp, &path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            ShrugError::AuthError(format!("Failed to save token file: {}", e))
        })?;

        Ok(())
    }

    /// Retrieve token from permission-restricted file.
    pub fn retrieve_token_file(&self, profile_name: &str) -> Option<String> {
        let path = self.token_file_path(profile_name);
        fs::read_to_string(&path).ok().map(|s| s.trim().to_string())
    }

    /// Try to retrieve token from OS keychain.
    pub fn retrieve_keychain(profile_name: &str) -> Option<String> {
        keyring::Entry::new("shrug", profile_name)
            .and_then(|e| e.get_password())
            .ok()
    }

    /// Retrieve token from encrypted file using password.
    pub fn retrieve_encrypted(
        &self,
        profile_name: &str,
        password: &str,
    ) -> Result<Option<String>, ShrugError> {
        let path = self.enc_file_path(profile_name);
        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path)
            .map_err(|e| ShrugError::AuthError(format!("Failed to read credential file: {}", e)))?;

        let blob: EncryptedBlob = serde_json::from_str(&json)
            .map_err(|e| ShrugError::AuthError(format!("Corrupted credential file: {}", e)))?;

        let token = decrypt_token(&blob, password)?;
        Ok(Some(token))
    }

    // --- OAuth Token Storage ---

    /// Store OAuth tokens via keychain (primary) or encrypted file (fallback).
    /// NEVER stores plaintext token files to disk.
    pub fn store_oauth_tokens(
        &self,
        profile_name: &str,
        tokens: &OAuthTokens,
    ) -> Result<(), ShrugError> {
        let json = serde_json::to_string(tokens).map_err(|e| {
            ShrugError::AuthError(format!("Failed to serialize OAuth tokens: {}", e))
        })?;

        // Try keychain first — verify with read-back to prevent silent data loss
        if Self::store_keychain_entry("shrug-oauth", profile_name, &json) {
            if Self::retrieve_keychain_entry("shrug-oauth", profile_name).is_some() {
                tracing::debug!("OAuth tokens stored in keychain");
                return Ok(());
            }
            tracing::debug!("Keychain store succeeded but verify-read failed, falling back");
        }

        // Fallback: encrypt and store as file
        tracing::debug!("Keychain unavailable for OAuth tokens, using encrypted file");
        let blob = encrypt_token(&json, &self.derive_oauth_file_key(profile_name))?;
        let enc_json = serde_json::to_string_pretty(&blob).map_err(|e| {
            ShrugError::AuthError(format!("Failed to serialize encrypted tokens: {}", e))
        })?;

        let path = self.oauth_token_file_path(profile_name);
        let tmp = self
            .credentials_dir
            .join(format!("{}.oauth.enc.tmp", profile_name));

        fs::write(&tmp, &enc_json).map_err(|e| {
            ShrugError::AuthError(format!("Failed to write OAuth token file: {}", e))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            ShrugError::AuthError(format!("Failed to save OAuth token file: {}", e))
        })?;

        Ok(())
    }

    /// Retrieve OAuth tokens from keychain or encrypted file.
    pub fn retrieve_oauth_tokens(
        &self,
        profile_name: &str,
    ) -> Result<Option<OAuthTokens>, ShrugError> {
        // Try keychain first
        if let Some(json) = Self::retrieve_keychain_entry("shrug-oauth", profile_name) {
            let tokens: OAuthTokens = serde_json::from_str(&json).map_err(|e| {
                ShrugError::AuthError(format!("Corrupted OAuth tokens in keychain: {}", e))
            })?;
            return Ok(Some(tokens));
        }

        // Try encrypted file
        let path = self.oauth_token_file_path(profile_name);
        if path.exists() {
            let enc_json = fs::read_to_string(&path).map_err(|e| {
                ShrugError::AuthError(format!("Failed to read OAuth token file: {}", e))
            })?;
            let blob: EncryptedBlob = serde_json::from_str(&enc_json)
                .map_err(|e| ShrugError::AuthError(format!("Corrupted OAuth token file: {}", e)))?;
            let json = decrypt_token(&blob, &self.derive_oauth_file_key(profile_name))?;
            let tokens: OAuthTokens = serde_json::from_str(&json)
                .map_err(|e| ShrugError::AuthError(format!("Corrupted OAuth token data: {}", e)))?;
            return Ok(Some(tokens));
        }

        Ok(None)
    }

    // --- OAuth Config Storage ---

    /// Store OAuth config (client_id + client_secret) via keychain or encrypted file.
    pub fn store_oauth_config(
        &self,
        profile_name: &str,
        config: &OAuthConfig,
    ) -> Result<(), ShrugError> {
        let json = serde_json::to_string(config).map_err(|e| {
            ShrugError::AuthError(format!("Failed to serialize OAuth config: {}", e))
        })?;

        // Try keychain first — verify with read-back; enables auto-refresh without password prompt
        if Self::store_keychain_entry("shrug-oauth-config", profile_name, &json) {
            if Self::retrieve_keychain_entry("shrug-oauth-config", profile_name).is_some() {
                tracing::debug!("OAuth config stored in keychain");
                return Ok(());
            }
            tracing::debug!("Keychain store succeeded but verify-read failed, falling back");
        }

        // Fallback: encrypt and store as file
        tracing::debug!("Keychain unavailable for OAuth config, using encrypted file");
        let blob = encrypt_token(&json, &self.derive_oauth_file_key(profile_name))?;
        let enc_json = serde_json::to_string_pretty(&blob).map_err(|e| {
            ShrugError::AuthError(format!("Failed to serialize encrypted config: {}", e))
        })?;

        let path = self.oauth_config_file_path(profile_name);
        let tmp = self
            .credentials_dir
            .join(format!("{}.oauth-config.enc.tmp", profile_name));

        fs::write(&tmp, &enc_json).map_err(|e| {
            ShrugError::AuthError(format!("Failed to write OAuth config file: {}", e))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            ShrugError::AuthError(format!("Failed to save OAuth config file: {}", e))
        })?;

        Ok(())
    }

    /// Retrieve OAuth config from keychain or encrypted file.
    pub fn retrieve_oauth_config(
        &self,
        profile_name: &str,
    ) -> Result<Option<OAuthConfig>, ShrugError> {
        // Try keychain first
        if let Some(json) = Self::retrieve_keychain_entry("shrug-oauth-config", profile_name) {
            let config: OAuthConfig = serde_json::from_str(&json).map_err(|e| {
                ShrugError::AuthError(format!("Corrupted OAuth config in keychain: {}", e))
            })?;
            return Ok(Some(config));
        }

        // Try encrypted file
        let path = self.oauth_config_file_path(profile_name);
        if path.exists() {
            let enc_json = fs::read_to_string(&path).map_err(|e| {
                ShrugError::AuthError(format!("Failed to read OAuth config file: {}", e))
            })?;
            let blob: EncryptedBlob = serde_json::from_str(&enc_json).map_err(|e| {
                ShrugError::AuthError(format!("Corrupted OAuth config file: {}", e))
            })?;
            let json = decrypt_token(&blob, &self.derive_oauth_file_key(profile_name))?;
            let config: OAuthConfig = serde_json::from_str(&json).map_err(|e| {
                ShrugError::AuthError(format!("Corrupted OAuth config data: {}", e))
            })?;
            return Ok(Some(config));
        }

        Ok(None)
    }

    // --- Credential Status ---

    /// Check if any credential exists (keychain, encrypted file, or OAuth tokens).
    pub fn has_credential(&self, profile_name: &str) -> Result<bool, ShrugError> {
        // Check API token keychain
        if Self::retrieve_keychain(profile_name).is_some() {
            return Ok(true);
        }
        // Check API token encrypted file
        if self.enc_file_path(profile_name).exists() {
            return Ok(true);
        }
        // Check API token file
        if self.token_file_path(profile_name).exists() {
            return Ok(true);
        }
        // Check OAuth tokens
        if Self::retrieve_keychain_entry("shrug-oauth", profile_name).is_some() {
            return Ok(true);
        }
        if self.oauth_token_file_path(profile_name).exists() {
            return Ok(true);
        }
        Ok(false)
    }

    /// Get the source of a stored credential, if any.
    pub fn credential_source(&self, profile_name: &str) -> Option<CredentialSource> {
        // API token sources
        if Self::retrieve_keychain(profile_name).is_some() {
            return Some(CredentialSource::Keychain);
        }
        if self.enc_file_path(profile_name).exists() {
            return Some(CredentialSource::EncryptedFile);
        }
        if self.token_file_path(profile_name).exists() {
            return Some(CredentialSource::TokenFile);
        }
        // OAuth token sources
        if Self::retrieve_keychain_entry("shrug-oauth", profile_name).is_some() {
            return Some(CredentialSource::Keychain);
        }
        if self.oauth_token_file_path(profile_name).exists() {
            return Some(CredentialSource::EncryptedFile);
        }
        None
    }

    /// Delete all credentials for a profile (keychain entries + encrypted files + OAuth).
    /// Silently ignores errors (credential may not exist in one or both).
    pub fn delete(&self, profile_name: &str) {
        // API token keychain
        if let Ok(entry) = keyring::Entry::new("shrug", profile_name) {
            let _ = entry.delete_credential();
        }
        // API token encrypted file
        let _ = fs::remove_file(self.enc_file_path(profile_name));
        // API token file
        let _ = fs::remove_file(self.token_file_path(profile_name));

        // OAuth tokens keychain + file
        if let Ok(entry) = keyring::Entry::new("shrug-oauth", profile_name) {
            let _ = entry.delete_credential();
        }
        let _ = fs::remove_file(self.oauth_token_file_path(profile_name));

        // OAuth config keychain + file
        if let Ok(entry) = keyring::Entry::new("shrug-oauth-config", profile_name) {
            let _ = entry.delete_credential();
        }
        let _ = fs::remove_file(self.oauth_config_file_path(profile_name));
    }

    // --- Credential Resolution ---

    /// Full credential resolution. Read-only — does NOT perform HTTP calls.
    ///
    /// For BasicAuth: env vars > keychain > encrypted file.
    /// For OAuth2: loads stored OAuth tokens and returns Bearer scheme.
    ///
    /// Call `ensure_fresh_tokens()` BEFORE this method for OAuth2 profiles.
    pub fn resolve(
        &self,
        profile: &Profile,
        encryption_password: Option<&str>,
    ) -> Result<Option<ResolvedCredential>, ShrugError> {
        let site = env::var("SHRUG_SITE").unwrap_or_else(|_| profile.site.clone());

        match profile.auth_type {
            AuthType::OAuth2 => {
                // OAuth2: load stored tokens (should be fresh — caller runs ensure_fresh_tokens first)
                if let Some(tokens) = self.retrieve_oauth_tokens(&profile.name)? {
                    return Ok(Some(ResolvedCredential {
                        site,
                        source: if Self::retrieve_keychain_entry("shrug-oauth", &profile.name)
                            .is_some()
                        {
                            CredentialSource::Keychain
                        } else {
                            CredentialSource::EncryptedFile
                        },
                        scheme: AuthScheme::Bearer {
                            access_token: tokens.access_token,
                        },
                    }));
                }
                Ok(None)
            }
            AuthType::BasicAuth => {
                let email = env::var("SHRUG_EMAIL").unwrap_or_else(|_| profile.email.clone());

                // 1. Environment variable token
                if let Ok(token) = env::var("SHRUG_API_TOKEN") {
                    if !token.is_empty() {
                        return Ok(Some(ResolvedCredential {
                            site,
                            source: CredentialSource::Environment,
                            scheme: AuthScheme::Basic {
                                email,
                                api_token: token,
                            },
                        }));
                    }
                }

                // 2. Keychain
                if let Some(token) = Self::retrieve_keychain(&profile.name) {
                    return Ok(Some(ResolvedCredential {
                        site,
                        source: CredentialSource::Keychain,
                        scheme: AuthScheme::Basic {
                            email,
                            api_token: token,
                        },
                    }));
                }

                // 3. Token file (permission-restricted, no password needed)
                if let Some(token) = self.retrieve_token_file(&profile.name) {
                    return Ok(Some(ResolvedCredential {
                        site,
                        source: CredentialSource::TokenFile,
                        scheme: AuthScheme::Basic {
                            email,
                            api_token: token,
                        },
                    }));
                }

                // 4. Encrypted file (requires password)
                if let Some(password) = encryption_password {
                    if let Some(token) = self.retrieve_encrypted(&profile.name, password)? {
                        return Ok(Some(ResolvedCredential {
                            site,
                            source: CredentialSource::EncryptedFile,
                            scheme: AuthScheme::Basic {
                                email,
                                api_token: token,
                            },
                        }));
                    }
                }

                Ok(None)
            }
        }
    }

    /// Ensure OAuth tokens are fresh before resolve(). Performs HTTP refresh if needed.
    ///
    /// Returns Ok(true) if tokens were refreshed, Ok(false) if no refresh needed.
    /// Returns Err if refresh fails (caller should direct user to re-authorize).
    pub fn ensure_fresh_tokens(&self, profile: &Profile) -> Result<bool, ShrugError> {
        if profile.auth_type != AuthType::OAuth2 {
            return Ok(false);
        }

        let tokens = match self.retrieve_oauth_tokens(&profile.name)? {
            Some(t) => t,
            None => return Ok(false), // No tokens to refresh
        };

        if !tokens.is_expired() {
            return Ok(false); // Still valid
        }

        tracing::info!(profile = %profile.name, "OAuth access token expired, refreshing");

        let config = self.retrieve_oauth_config(&profile.name)?.ok_or_else(|| {
            ShrugError::AuthError(format!(
                "OAuth config not found for profile '{}'. Run `shrug auth setup` to configure.",
                profile.name
            ))
        })?;

        match oauth::refresh_tokens(&config, &tokens.refresh_token) {
            Ok(new_tokens) => {
                self.store_oauth_tokens(&profile.name, &new_tokens)?;
                tracing::info!(profile = %profile.name, "OAuth tokens refreshed successfully");
                Ok(true)
            }
            Err(e) => Err(ShrugError::AuthError(format!(
                "OAuth token expired and refresh failed: {}. Run `shrug auth login --profile {}` to re-authorize.",
                e, profile.name
            ))),
        }
    }

    /// Check if an encrypted file exists for this profile (needs password to retrieve).
    pub fn has_encrypted_credential(&self, profile_name: &str) -> bool {
        self.enc_file_path(profile_name).exists()
    }

    // --- Private helpers ---

    fn enc_file_path(&self, profile_name: &str) -> PathBuf {
        self.credentials_dir.join(format!("{}.enc", profile_name))
    }

    fn token_file_path(&self, profile_name: &str) -> PathBuf {
        self.credentials_dir.join(format!("{}.token", profile_name))
    }

    fn oauth_token_file_path(&self, profile_name: &str) -> PathBuf {
        self.credentials_dir
            .join(format!("{}.oauth.enc", profile_name))
    }

    fn oauth_config_file_path(&self, profile_name: &str) -> PathBuf {
        self.credentials_dir
            .join(format!("{}.oauth-config.enc", profile_name))
    }

    /// Store a value in keychain under a given service name.
    /// Verifies the write persisted by reading it back.
    fn store_keychain_entry(service: &str, profile_name: &str, value: &str) -> bool {
        let Ok(entry) = keyring::Entry::new(service, profile_name) else {
            return false;
        };
        if entry.set_password(value).is_err() {
            return false;
        }
        Self::retrieve_keychain_entry(service, profile_name).is_some()
    }

    /// Retrieve a value from keychain under a given service name.
    fn retrieve_keychain_entry(service: &str, profile_name: &str) -> Option<String> {
        keyring::Entry::new(service, profile_name)
            .and_then(|e| e.get_password())
            .ok()
    }

    /// Derive a deterministic encryption key for OAuth file fallback.
    /// Uses the profile name + a fixed salt. This is a fallback when keychain is unavailable —
    /// it provides encryption-at-rest but the key is derivable, so it's weaker than keychain.
    /// For stronger security, users should ensure their OS keychain is available.
    fn derive_oauth_file_key(&self, profile_name: &str) -> String {
        format!("shrug-oauth-{}-fallback", profile_name)
    }
}

// --- Encryption internals ---

#[derive(Serialize, Deserialize)]
struct EncryptedBlob {
    salt: String,
    nonce: String,
    ciphertext: String,
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], ShrugError> {
    use argon2::Argon2;

    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| ShrugError::AuthError(format!("Key derivation failed: {}", e)))?;
    Ok(key)
}

fn encrypt_token(token: &str, password: &str) -> Result<EncryptedBlob, ShrugError> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::{engine::general_purpose::STANDARD, Engine};
    use rand::RngCore;

    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ShrugError::AuthError(format!("Cipher init failed: {}", e)))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, token.as_bytes())
        .map_err(|e| ShrugError::AuthError(format!("Encryption failed: {}", e)))?;

    Ok(EncryptedBlob {
        salt: STANDARD.encode(salt),
        nonce: STANDARD.encode(nonce_bytes),
        ciphertext: STANDARD.encode(ciphertext),
    })
}

fn decrypt_token(blob: &EncryptedBlob, password: &str) -> Result<String, ShrugError> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::{engine::general_purpose::STANDARD, Engine};

    let salt = STANDARD.decode(&blob.salt).map_err(|e| {
        ShrugError::AuthError(format!("Corrupted credential file (bad salt): {}", e))
    })?;
    let nonce_bytes = STANDARD.decode(&blob.nonce).map_err(|e| {
        ShrugError::AuthError(format!("Corrupted credential file (bad nonce): {}", e))
    })?;
    let ciphertext = STANDARD.decode(&blob.ciphertext).map_err(|e| {
        ShrugError::AuthError(format!("Corrupted credential file (bad ciphertext): {}", e))
    })?;

    if nonce_bytes.len() != 12 {
        return Err(ShrugError::AuthError(
            "Corrupted credential file (invalid nonce length)".into(),
        ));
    }

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ShrugError::AuthError(format!("Cipher init failed: {}", e)))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| {
        ShrugError::AuthError("Decryption failed (wrong password or corrupted file)".into())
    })?;

    String::from_utf8(plaintext)
        .map_err(|e| ShrugError::AuthError(format!("Decrypted data is not valid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Env var tests must run serially to avoid race conditions.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn make_store(dir: &tempfile::TempDir) -> CredentialStore {
        CredentialStore::new(dir.path().to_path_buf()).unwrap()
    }

    fn make_profile(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            site: "https://test.atlassian.net".to_string(),
            email: "user@example.com".to_string(),
            auth_type: crate::auth::profile::AuthType::default(),
        }
    }

    fn make_oauth_profile(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            site: "https://test.atlassian.net".to_string(),
            email: "user@example.com".to_string(),
            auth_type: AuthType::OAuth2,
        }
    }

    #[test]
    fn encrypted_store_retrieve_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        store
            .store_encrypted("test-profile", "my-secret-token", "password123")
            .unwrap();

        let result = store
            .retrieve_encrypted("test-profile", "password123")
            .unwrap();
        assert_eq!(result, Some("my-secret-token".to_string()));
    }

    #[test]
    fn encrypted_wrong_password_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        store
            .store_encrypted("test-profile", "my-secret-token", "password123")
            .unwrap();

        let result = store.retrieve_encrypted("test-profile", "wrong-password");
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("wrong password") || msg.contains("Decryption failed"),
            "Expected decryption error: {msg}"
        );
    }

    #[test]
    fn encrypted_delete_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        store
            .store_encrypted("test-profile", "token", "pass")
            .unwrap();
        assert!(store.has_encrypted_credential("test-profile"));

        store.delete("test-profile");
        assert!(!store.has_encrypted_credential("test-profile"));
    }

    #[test]
    fn has_credential_checks_encrypted_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-hascred-encfile";
        store.delete(name);

        assert!(!store.has_credential(name).unwrap());

        store.store_encrypted(name, "token", "pass").unwrap();
        assert!(store.has_credential(name).unwrap());

        store.delete(name);
    }

    #[test]
    fn encrypted_corrupted_file_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        let path = dir.path().join("credentials").join("bad.enc");
        fs::write(&path, "not valid json").unwrap();

        let result = store.retrieve_encrypted("bad", "password");
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("Corrupted"),
            "Expected corruption error: {msg}"
        );
    }

    #[test]
    fn encrypted_nonexistent_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);

        let result = store.retrieve_encrypted("nope", "password").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn credential_source_display() {
        assert_eq!(format!("{}", CredentialSource::Keychain), "keychain");
        assert_eq!(
            format!("{}", CredentialSource::EncryptedFile),
            "encrypted-file"
        );
        assert_eq!(format!("{}", CredentialSource::Environment), "environment");
    }

    #[test]
    fn resolve_env_vars_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-env-override";
        store.delete(name);
        let profile = make_profile(name);

        let orig_token = env::var("SHRUG_API_TOKEN").ok();
        let orig_email = env::var("SHRUG_EMAIL").ok();
        let orig_site = env::var("SHRUG_SITE").ok();

        env::set_var("SHRUG_API_TOKEN", "env-token-123");
        env::set_var("SHRUG_EMAIL", "env@example.com");
        env::set_var("SHRUG_SITE", "https://env.atlassian.net");

        let result = store.resolve(&profile, None).unwrap().unwrap();

        match orig_token {
            Some(v) => env::set_var("SHRUG_API_TOKEN", v),
            None => env::remove_var("SHRUG_API_TOKEN"),
        }
        match orig_email {
            Some(v) => env::set_var("SHRUG_EMAIL", v),
            None => env::remove_var("SHRUG_EMAIL"),
        }
        match orig_site {
            Some(v) => env::set_var("SHRUG_SITE", v),
            None => env::remove_var("SHRUG_SITE"),
        }

        match &result.scheme {
            AuthScheme::Basic { email, api_token } => {
                assert_eq!(api_token, "env-token-123");
                assert_eq!(email, "env@example.com");
            }
            _ => panic!("Expected Basic scheme"),
        }
        assert_eq!(result.site, "https://env.atlassian.net");
        assert_eq!(result.source, CredentialSource::Environment);
    }

    #[test]
    fn resolve_env_skips_backend() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-env-skips";
        store.delete(name);
        let profile = make_profile(name);

        store.store_encrypted(name, "file-token", "pass").unwrap();

        let orig = env::var("SHRUG_API_TOKEN").ok();
        env::set_var("SHRUG_API_TOKEN", "env-token");

        let result = store.resolve(&profile, Some("pass")).unwrap().unwrap();

        match orig {
            Some(v) => env::set_var("SHRUG_API_TOKEN", v),
            None => env::remove_var("SHRUG_API_TOKEN"),
        }

        assert_eq!(result.source, CredentialSource::Environment);
        match &result.scheme {
            AuthScheme::Basic { api_token, .. } => assert_eq!(api_token, "env-token"),
            _ => panic!("Expected Basic scheme"),
        }
    }

    #[test]
    fn resolve_returns_none_when_no_credentials() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-resolve-none-cred";
        store.delete(name);
        let profile = make_profile(name);

        let orig = env::var("SHRUG_API_TOKEN").ok();
        env::remove_var("SHRUG_API_TOKEN");

        let result = store.resolve(&profile, None).unwrap();

        if let Some(v) = orig {
            env::set_var("SHRUG_API_TOKEN", v);
        }

        assert!(result.is_none());
    }

    #[test]
    fn resolve_encrypted_file_with_password() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-resolve-encfile";
        store.delete(name);
        let profile = make_profile(name);

        let orig = env::var("SHRUG_API_TOKEN").ok();
        env::remove_var("SHRUG_API_TOKEN");

        store
            .store_encrypted(name, "encrypted-token", "mypass")
            .unwrap();

        let result = store.resolve(&profile, Some("mypass")).unwrap().unwrap();

        if let Some(v) = orig {
            env::set_var("SHRUG_API_TOKEN", v);
        }

        match &result.scheme {
            AuthScheme::Basic { api_token, .. } => assert_eq!(api_token, "encrypted-token"),
            _ => panic!("Expected Basic scheme"),
        }
        assert_eq!(result.source, CredentialSource::EncryptedFile);
    }

    #[test]
    fn credential_source_equality() {
        assert_eq!(CredentialSource::Keychain, CredentialSource::Keychain);
        assert_ne!(CredentialSource::Keychain, CredentialSource::Environment);
    }

    #[test]
    fn has_credential_returns_result() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-hascred-result";
        store.delete(name);
        let result: Result<bool, ShrugError> = store.has_credential(name);
        assert!(result.is_ok());
    }

    // --- OAuth Token Storage Tests ---

    #[test]
    fn oauth_tokens_store_retrieve_roundtrip_encrypted() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-oauth-roundtrip";
        store.delete(name);

        let tokens = OAuthTokens {
            access_token: "access-123".to_string(),
            refresh_token: "refresh-456".to_string(),
            expires_at: None,
            scopes: vec!["read:jira-work".to_string()],
        };

        store.store_oauth_tokens(name, &tokens).unwrap();

        let retrieved = store.retrieve_oauth_tokens(name).unwrap().unwrap();
        assert_eq!(retrieved.access_token, "access-123");
        assert_eq!(retrieved.refresh_token, "refresh-456");

        store.delete(name);
    }

    #[test]
    fn oauth_tokens_no_plaintext_json_files() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-oauth-plaintext";
        store.delete(name);

        let tokens = OAuthTokens {
            access_token: "secret".to_string(),
            refresh_token: "also-secret".to_string(),
            expires_at: None,
            scopes: vec![],
        };

        store.store_oauth_tokens(name, &tokens).unwrap();

        // Verify no .oauth.json file exists (plaintext)
        let plaintext_path = dir
            .path()
            .join("credentials")
            .join(format!("{}.oauth.json", name));
        assert!(
            !plaintext_path.exists(),
            "Plaintext .oauth.json should NOT exist"
        );

        store.delete(name);
    }

    #[test]
    fn oauth_config_store_retrieve_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-oauth-config";
        store.delete(name);

        let config = OAuthConfig {
            client_id: "client-id-123".to_string(),
            client_secret: "secret-456".to_string(),
            redirect_port: 9000,
        };

        store.store_oauth_config(name, &config).unwrap();

        let retrieved = store.retrieve_oauth_config(name).unwrap().unwrap();
        assert_eq!(retrieved.client_id, "client-id-123");
        assert_eq!(retrieved.client_secret, "secret-456");

        store.delete(name);
    }

    #[test]
    fn delete_cleans_up_oauth_files() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-delete-cleanup";
        store.delete(name);

        let tokens = OAuthTokens {
            access_token: "a".to_string(),
            refresh_token: "b".to_string(),
            expires_at: None,
            scopes: vec![],
        };
        let config = OAuthConfig {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
            redirect_port: 8456,
        };

        store.store_oauth_tokens(name, &tokens).unwrap();
        store.store_oauth_config(name, &config).unwrap();

        store.delete(name);

        assert!(store.retrieve_oauth_tokens(name).unwrap().is_none());
        assert!(store.retrieve_oauth_config(name).unwrap().is_none());
    }

    #[test]
    fn resolve_oauth2_profile_returns_bearer_scheme() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-resolve-bearer";
        let profile = make_oauth_profile(name);
        store.delete(name);

        let tokens = OAuthTokens {
            access_token: "bearer-token-123".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            scopes: vec![],
        };

        store.store_oauth_tokens(name, &tokens).unwrap();

        let result = store.resolve(&profile, None).unwrap().unwrap();
        match &result.scheme {
            AuthScheme::Bearer { access_token } => {
                assert_eq!(access_token, "bearer-token-123");
            }
            _ => panic!("Expected Bearer scheme for OAuth2 profile"),
        }

        store.delete(name);
    }

    #[test]
    fn resolve_oauth2_no_tokens_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-resolve-none";
        let profile = make_oauth_profile(name);
        store.delete(name);

        let result = store.resolve(&profile, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn ensure_fresh_tokens_skips_basic_auth() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-fresh-basic";
        let profile = make_profile(name);
        store.delete(name);

        let result = store.ensure_fresh_tokens(&profile).unwrap();
        assert!(!result);
    }

    #[test]
    fn ensure_fresh_tokens_skips_when_not_expired() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-fresh-notexpired";
        let profile = make_oauth_profile(name);
        store.delete(name);

        let tokens = OAuthTokens {
            access_token: "still-valid".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            scopes: vec![],
        };
        store.store_oauth_tokens(name, &tokens).unwrap();

        let result = store.ensure_fresh_tokens(&profile).unwrap();
        assert!(!result);

        store.delete(name);
    }

    #[test]
    fn has_credential_detects_oauth_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let name = "test-hascred-oauth";
        store.delete(name);

        assert!(!store.has_credential(name).unwrap());

        let tokens = OAuthTokens {
            access_token: "a".to_string(),
            refresh_token: "b".to_string(),
            expires_at: None,
            scopes: vec![],
        };
        store.store_oauth_tokens(name, &tokens).unwrap();

        assert!(store.has_credential(name).unwrap());

        store.delete(name);
    }
}
