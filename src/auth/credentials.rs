use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::auth::profile::Profile;
use crate::error::ShrugError;

/// Source of a resolved credential.
#[derive(Debug, Clone, PartialEq)]
pub enum CredentialSource {
    /// From OS keychain (Windows Credential Manager, macOS Keychain, Linux Secret Service)
    Keychain,
    /// From encrypted file fallback
    EncryptedFile,
    /// From environment variables
    Environment,
}

impl std::fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialSource::Keychain => write!(f, "keychain"),
            CredentialSource::EncryptedFile => write!(f, "encrypted-file"),
            CredentialSource::Environment => write!(f, "environment"),
        }
    }
}

/// A fully resolved credential ready for use in HTTP requests.
#[derive(Debug, Clone)]
pub struct ResolvedCredential {
    pub email: String,
    pub api_token: String,
    pub site: String,
    pub source: CredentialSource,
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

    /// Store token in OS keychain. Returns true if stored, false if keychain unavailable.
    pub fn store_keychain(profile_name: &str, token: &str) -> bool {
        match keyring::Entry::new("shrug", profile_name) {
            Ok(entry) => entry.set_password(token).is_ok(),
            Err(_) => false,
        }
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

    /// Check if any credential exists (keychain or encrypted file).
    pub fn has_credential(&self, profile_name: &str) -> Result<bool, ShrugError> {
        // Check keychain first
        if Self::retrieve_keychain(profile_name).is_some() {
            return Ok(true);
        }
        // Check encrypted file
        Ok(self.enc_file_path(profile_name).exists())
    }

    /// Get the source of a stored credential, if any.
    pub fn credential_source(&self, profile_name: &str) -> Option<CredentialSource> {
        if Self::retrieve_keychain(profile_name).is_some() {
            return Some(CredentialSource::Keychain);
        }
        if self.enc_file_path(profile_name).exists() {
            return Some(CredentialSource::EncryptedFile);
        }
        None
    }

    /// Delete credential from both keychain and encrypted file.
    /// Silently ignores errors (credential may not exist in one or both).
    pub fn delete(&self, profile_name: &str) {
        // Try keychain
        if let Ok(entry) = keyring::Entry::new("shrug", profile_name) {
            let _ = entry.delete_credential();
        }
        // Try encrypted file
        let _ = fs::remove_file(self.enc_file_path(profile_name));
    }

    /// Full credential resolution: env vars > keychain > encrypted file.
    ///
    /// Pass `encryption_password` if the user has provided one for encrypted file access.
    /// If None and only encrypted file credentials exist, returns None.
    pub fn resolve(
        &self,
        profile: &Profile,
        encryption_password: Option<&str>,
    ) -> Result<Option<ResolvedCredential>, ShrugError> {
        let email = env::var("SHRUG_EMAIL").unwrap_or_else(|_| profile.email.clone());
        let site = env::var("SHRUG_SITE").unwrap_or_else(|_| profile.site.clone());

        // 1. Environment variable token
        if let Ok(token) = env::var("SHRUG_API_TOKEN") {
            if !token.is_empty() {
                return Ok(Some(ResolvedCredential {
                    email,
                    api_token: token,
                    site,
                    source: CredentialSource::Environment,
                }));
            }
        }

        // 2. Keychain
        if let Some(token) = Self::retrieve_keychain(&profile.name) {
            return Ok(Some(ResolvedCredential {
                email,
                api_token: token,
                site,
                source: CredentialSource::Keychain,
            }));
        }

        // 3. Encrypted file (requires password)
        if let Some(password) = encryption_password {
            if let Some(token) = self.retrieve_encrypted(&profile.name, password)? {
                return Ok(Some(ResolvedCredential {
                    email,
                    api_token: token,
                    site,
                    source: CredentialSource::EncryptedFile,
                }));
            }
        }

        Ok(None)
    }

    /// Check if an encrypted file exists for this profile (needs password to retrieve).
    pub fn has_encrypted_credential(&self, profile_name: &str) -> bool {
        self.enc_file_path(profile_name).exists()
    }

    fn enc_file_path(&self, profile_name: &str) -> PathBuf {
        self.credentials_dir.join(format!("{}.enc", profile_name))
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

        assert_eq!(store.has_credential("test-profile").unwrap(), false);

        store
            .store_encrypted("test-profile", "token", "pass")
            .unwrap();
        assert_eq!(store.has_credential("test-profile").unwrap(), true);
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
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let profile = make_profile("test");

        // Save/restore env vars
        let orig_token = env::var("SHRUG_API_TOKEN").ok();
        let orig_email = env::var("SHRUG_EMAIL").ok();
        let orig_site = env::var("SHRUG_SITE").ok();

        env::set_var("SHRUG_API_TOKEN", "env-token-123");
        env::set_var("SHRUG_EMAIL", "env@example.com");
        env::set_var("SHRUG_SITE", "https://env.atlassian.net");

        let result = store.resolve(&profile, None).unwrap().unwrap();

        // Restore
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

        assert_eq!(result.api_token, "env-token-123");
        assert_eq!(result.email, "env@example.com");
        assert_eq!(result.site, "https://env.atlassian.net");
        assert_eq!(result.source, CredentialSource::Environment);
    }

    #[test]
    fn resolve_env_skips_backend() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let profile = make_profile("test");

        // Store encrypted credential
        store.store_encrypted("test", "file-token", "pass").unwrap();

        // Set env var — should take precedence
        let orig = env::var("SHRUG_API_TOKEN").ok();
        env::set_var("SHRUG_API_TOKEN", "env-token");

        let result = store.resolve(&profile, Some("pass")).unwrap().unwrap();

        match orig {
            Some(v) => env::set_var("SHRUG_API_TOKEN", v),
            None => env::remove_var("SHRUG_API_TOKEN"),
        }

        assert_eq!(result.source, CredentialSource::Environment);
        assert_eq!(result.api_token, "env-token");
    }

    #[test]
    fn resolve_returns_none_when_no_credentials() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let profile = make_profile("test");

        // Ensure env var is not set
        let orig = env::var("SHRUG_API_TOKEN").ok();
        env::remove_var("SHRUG_API_TOKEN");

        let result = store.resolve(&profile, None).unwrap();

        match orig {
            Some(v) => env::set_var("SHRUG_API_TOKEN", v),
            None => {}
        }

        assert!(result.is_none());
    }

    #[test]
    fn resolve_encrypted_file_with_password() {
        let dir = tempfile::tempdir().unwrap();
        let store = make_store(&dir);
        let profile = make_profile("test");

        // Ensure env var is not set
        let orig = env::var("SHRUG_API_TOKEN").ok();
        env::remove_var("SHRUG_API_TOKEN");

        store
            .store_encrypted("test", "encrypted-token", "mypass")
            .unwrap();

        let result = store.resolve(&profile, Some("mypass")).unwrap().unwrap();

        match orig {
            Some(v) => env::set_var("SHRUG_API_TOKEN", v),
            None => {}
        }

        assert_eq!(result.api_token, "encrypted-token");
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
        // Verify it returns Result, not bool
        let result: Result<bool, ShrugError> = store.has_credential("test");
        assert!(result.is_ok());
    }
}
