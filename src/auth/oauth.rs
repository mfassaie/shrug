use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ShrugError;

/// Default scopes for Atlassian OAuth 2.0 3LO.
const DEFAULT_SCOPES: &str = "read:jira-work write:jira-work read:jira-user \
    read:confluence-content.all write:confluence-content \
    read:confluence-space.summary read:bitbucket-user \
    read:servicedesk-request write:servicedesk-request offline_access";

/// OAuth 2.0 token set returned from Atlassian auth server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

impl OAuthTokens {
    /// Check if the access token is expired (with 60-second safety margin).
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expiry) => Utc::now() + chrono::Duration::seconds(60) >= expiry,
            None => false, // No expiry info — assume valid
        }
    }
}

/// OAuth 2.0 client configuration (client_id + client_secret from Atlassian developer console).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default = "default_redirect_port")]
    pub redirect_port: u16,
}

fn default_redirect_port() -> u16 {
    8456
}

/// PKCE verifier + challenge pair.
pub struct PkcePair {
    pub verifier: String,
    pub challenge: String,
}

/// Generate a PKCE (Proof Key for Code Exchange) verifier and challenge pair.
///
/// - verifier: 64-char random URL-safe base64 string (within RFC 7636 43-128 range)
/// - challenge: base64url(sha256(verifier)) with no padding
pub fn generate_pkce_pair() -> PkcePair {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use rand::RngCore;
    use sha2::{Digest, Sha256};

    // Generate 48 random bytes → 64 base64url chars
    let mut random_bytes = [0u8; 48];
    rand::rngs::OsRng.fill_bytes(&mut random_bytes);
    let verifier = URL_SAFE_NO_PAD.encode(random_bytes);

    // challenge = base64url(sha256(verifier))
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let challenge = URL_SAFE_NO_PAD.encode(hash);

    PkcePair {
        verifier,
        challenge,
    }
}

/// Build the Atlassian authorization URL and return it with the PKCE verifier and state.
pub fn start_auth_flow(config: &OAuthConfig) -> Result<(String, String, String), ShrugError> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use rand::RngCore;

    let pkce = generate_pkce_pair();

    // Generate random state parameter
    let mut state_bytes = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut state_bytes);
    let state = URL_SAFE_NO_PAD.encode(state_bytes);

    let redirect_uri = format!("http://127.0.0.1:{}/callback", config.redirect_port);

    let mut url = url::Url::parse("https://auth.atlassian.com/authorize")
        .map_err(|e| ShrugError::AuthError(format!("Failed to build authorization URL: {}", e)))?;

    url.query_pairs_mut()
        .append_pair("audience", "api.atlassian.com")
        .append_pair("client_id", &config.client_id)
        .append_pair("scope", DEFAULT_SCOPES)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("state", &state)
        .append_pair("response_type", "code")
        .append_pair("prompt", "consent")
        .append_pair("code_challenge", &pkce.challenge)
        .append_pair("code_challenge_method", "S256");

    Ok((url.to_string(), pkce.verifier, state))
}

/// Start a local TCP server on 127.0.0.1:{port} and wait for the OAuth callback.
///
/// Returns the authorization code on success.
/// Handles error callbacks (?error=...) with actionable error messages.
/// Times out after 120 seconds.
pub fn await_callback(port: u16, expected_state: &str) -> Result<String, ShrugError> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).map_err(|e| {
        ShrugError::AuthError(format!(
            "Failed to start callback server on {}: {}. Is port {} already in use?",
            addr, e, port
        ))
    })?;

    // Set timeout for the listener
    listener
        .set_nonblocking(false)
        .map_err(|e| ShrugError::AuthError(format!("Failed to configure listener: {}", e)))?;

    // Accept one connection with timeout
    let timeout = Duration::from_secs(120);
    let start = std::time::Instant::now();

    // Use a polling approach with short accept timeouts
    loop {
        if start.elapsed() >= timeout {
            return Err(ShrugError::AuthError(
                "OAuth callback timed out after 120 seconds. Re-run `shrug auth login` to try again.".into(),
            ));
        }

        // Set a short SO_TIMEOUT so we can check elapsed time
        let _ = listener.set_nonblocking(true);
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(false);
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                return handle_callback_connection(stream, expected_state);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                return Err(ShrugError::AuthError(format!(
                    "Failed to accept callback connection: {}",
                    e
                )));
            }
        }
    }
}

fn handle_callback_connection(
    stream: std::net::TcpStream,
    expected_state: &str,
) -> Result<String, ShrugError> {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|e| ShrugError::AuthError(format!("Failed to read callback request: {}", e)))?;

    // Parse GET /callback?params HTTP/1.1
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| ShrugError::AuthError("Invalid callback request format".into()))?;

    let parsed = url::Url::parse(&format!("http://localhost{}", path))
        .map_err(|e| ShrugError::AuthError(format!("Failed to parse callback URL: {}", e)))?;

    let params: std::collections::HashMap<String, String> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // Check for error response first
    if let Some(error) = params.get("error") {
        let description = params
            .get("error_description")
            .map(|d| d.as_str())
            .unwrap_or("No description provided");

        // Send error response to browser
        let html = format!(
            "<html><body><h2>Authorization Failed</h2><p>{}: {}</p>\
             <p>You can close this tab.</p></body></html>",
            error, description
        );
        send_http_response(&stream, "400 Bad Request", &html);

        return Err(ShrugError::AuthError(format!(
            "Authorization denied: {} — {}. Re-run `shrug auth login` to try again.",
            error, description
        )));
    }

    // Validate state parameter
    let state = params.get("state").ok_or_else(|| {
        send_http_response(
            &stream,
            "400 Bad Request",
            "<html><body><h2>Missing state parameter</h2></body></html>",
        );
        ShrugError::AuthError("OAuth callback missing state parameter".into())
    })?;

    if state != expected_state {
        send_http_response(
            &stream,
            "400 Bad Request",
            "<html><body><h2>Invalid state</h2></body></html>",
        );
        return Err(ShrugError::AuthError(
            "OAuth state mismatch — possible CSRF attack. Re-run `shrug auth login` to try again."
                .into(),
        ));
    }

    // Extract authorization code
    let code = params.get("code").ok_or_else(|| {
        send_http_response(
            &stream,
            "400 Bad Request",
            "<html><body><h2>Missing authorization code</h2></body></html>",
        );
        ShrugError::AuthError("OAuth callback missing authorization code".into())
    })?;

    // Send success response
    send_http_response(
        &stream,
        "200 OK",
        "<html><body><h2>Authorization Successful</h2>\
         <p>You can close this tab and return to the terminal.</p></body></html>",
    );

    Ok(code.clone())
}

fn send_http_response(stream: &std::net::TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.len(),
        body
    );
    let mut writer = stream;
    let _ = writer.write_all(response.as_bytes());
    let _ = writer.flush();
}

/// Exchange an authorization code for OAuth tokens.
pub fn exchange_code(
    config: &OAuthConfig,
    code: &str,
    verifier: &str,
) -> Result<OAuthTokens, ShrugError> {
    let redirect_uri = format!("http://127.0.0.1:{}/callback", config.redirect_port);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://auth.atlassian.com/oauth/token")
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "client_id": config.client_id,
            "client_secret": config.client_secret,
            "code": code,
            "redirect_uri": redirect_uri,
            "code_verifier": verifier,
        }))
        .send()
        .map_err(|e| ShrugError::AuthError(format!("Token exchange request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(ShrugError::AuthError(format!(
            "Token exchange failed (HTTP {}): {}",
            status, body
        )));
    }

    parse_token_response(response)
}

/// Refresh OAuth tokens using a refresh token.
pub fn refresh_tokens(
    config: &OAuthConfig,
    refresh_token: &str,
) -> Result<OAuthTokens, ShrugError> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://auth.atlassian.com/oauth/token")
        .json(&serde_json::json!({
            "grant_type": "refresh_token",
            "client_id": config.client_id,
            "client_secret": config.client_secret,
            "refresh_token": refresh_token,
        }))
        .send()
        .map_err(|e| ShrugError::AuthError(format!("Token refresh request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(ShrugError::AuthError(format!(
            "Token refresh failed (HTTP {}): {}",
            status, body
        )));
    }

    parse_token_response(response)
}

fn parse_token_response(response: reqwest::blocking::Response) -> Result<OAuthTokens, ShrugError> {
    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<i64>,
        scope: Option<String>,
    }

    let body: TokenResponse = response
        .json()
        .map_err(|e| ShrugError::AuthError(format!("Failed to parse token response: {}", e)))?;

    let expires_at = body
        .expires_in
        .map(|secs| Utc::now() + chrono::Duration::seconds(secs));

    let scopes = body
        .scope
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default();

    Ok(OAuthTokens {
        access_token: body.access_token,
        refresh_token: body.refresh_token.unwrap_or_default(),
        expires_at,
        scopes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_length_is_valid() {
        let pair = generate_pkce_pair();
        // RFC 7636: verifier must be 43-128 chars
        assert!(
            pair.verifier.len() >= 43 && pair.verifier.len() <= 128,
            "Verifier length {} not in 43-128 range",
            pair.verifier.len()
        );
    }

    #[test]
    fn pkce_verifier_is_url_safe_base64() {
        let pair = generate_pkce_pair();
        // URL-safe base64 chars: A-Z, a-z, 0-9, -, _
        for ch in pair.verifier.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '-' || ch == '_',
                "Invalid verifier char: '{}'",
                ch
            );
        }
    }

    #[test]
    fn pkce_challenge_is_sha256_of_verifier() {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        use base64::Engine;
        use sha2::{Digest, Sha256};

        let pair = generate_pkce_pair();

        let mut hasher = Sha256::new();
        hasher.update(pair.verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(hasher.finalize());

        assert_eq!(pair.challenge, expected);
    }

    #[test]
    fn pkce_pairs_are_unique() {
        let pair1 = generate_pkce_pair();
        let pair2 = generate_pkce_pair();
        assert_ne!(pair1.verifier, pair2.verifier);
        assert_ne!(pair1.challenge, pair2.challenge);
    }

    #[test]
    fn start_auth_flow_returns_valid_url() {
        let config = OAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-secret".to_string(),
            redirect_port: 8456,
        };

        let (url, verifier, state) = start_auth_flow(&config).unwrap();

        assert!(url.starts_with("https://auth.atlassian.com/authorize?"));
        assert!(url.contains("client_id=test-client-id"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("audience=api.atlassian.com"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("127.0.0.1%3A8456"));
        assert!(!verifier.is_empty());
        assert!(!state.is_empty());
    }

    #[test]
    fn start_auth_flow_state_is_unique() {
        let config = OAuthConfig {
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
            redirect_port: 8456,
        };

        let (_, _, state1) = start_auth_flow(&config).unwrap();
        let (_, _, state2) = start_auth_flow(&config).unwrap();
        assert_ne!(state1, state2);
    }

    #[test]
    fn oauth_tokens_not_expired_when_future() {
        let tokens = OAuthTokens {
            access_token: "test".to_string(),
            refresh_token: "test".to_string(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            scopes: vec![],
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn oauth_tokens_expired_when_past() {
        let tokens = OAuthTokens {
            access_token: "test".to_string(),
            refresh_token: "test".to_string(),
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
            scopes: vec![],
        };
        assert!(tokens.is_expired());
    }

    #[test]
    fn oauth_tokens_expired_within_safety_margin() {
        // Token expires in 30 seconds — within the 60s safety margin
        let tokens = OAuthTokens {
            access_token: "test".to_string(),
            refresh_token: "test".to_string(),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(30)),
            scopes: vec![],
        };
        assert!(tokens.is_expired());
    }

    #[test]
    fn oauth_tokens_not_expired_when_no_expiry() {
        let tokens = OAuthTokens {
            access_token: "test".to_string(),
            refresh_token: "test".to_string(),
            expires_at: None,
            scopes: vec![],
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn oauth_tokens_serialization_roundtrip() {
        let tokens = OAuthTokens {
            access_token: "access-123".to_string(),
            refresh_token: "refresh-456".to_string(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            scopes: vec!["read:jira-work".to_string(), "write:jira-work".to_string()],
        };

        let json = serde_json::to_string(&tokens).unwrap();
        let deserialized: OAuthTokens = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.access_token, "access-123");
        assert_eq!(deserialized.refresh_token, "refresh-456");
        assert_eq!(deserialized.scopes.len(), 2);
    }

    #[test]
    fn oauth_config_serialization_roundtrip() {
        let config = OAuthConfig {
            client_id: "client-id-123".to_string(),
            client_secret: "secret-456".to_string(),
            redirect_port: 9000,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OAuthConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.client_id, "client-id-123");
        assert_eq!(deserialized.client_secret, "secret-456");
        assert_eq!(deserialized.redirect_port, 9000);
    }

    #[test]
    fn oauth_config_default_port() {
        let json = r#"{"client_id":"id","client_secret":"secret"}"#;
        let config: OAuthConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.redirect_port, 8456);
    }

    #[test]
    fn callback_parses_error_response() {
        use std::io::Write;
        use std::net::TcpStream;

        // Start listener
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        // Simulate error callback in background
        let handle = std::thread::spawn(move || {
            let mut client = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
            client
                .write_all(
                    b"GET /callback?error=access_denied&error_description=User+denied+access&state=test-state HTTP/1.1\r\nHost: localhost\r\n\r\n",
                )
                .unwrap();
            // Read response
            let _ = client.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = [0u8; 1024];
            let _ = std::io::Read::read(&mut client, &mut buf);
        });

        let (stream, _) = listener.accept().unwrap();
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let result = handle_callback_connection(stream, "test-state");

        handle.join().unwrap();

        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("access_denied"),
            "Expected access_denied: {msg}"
        );
        assert!(
            msg.contains("User denied access"),
            "Expected description: {msg}"
        );
    }

    #[test]
    fn callback_rejects_state_mismatch() {
        use std::io::Write;
        use std::net::TcpStream;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = std::thread::spawn(move || {
            let mut client = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
            client
                .write_all(
                    b"GET /callback?code=auth-code&state=wrong-state HTTP/1.1\r\nHost: localhost\r\n\r\n",
                )
                .unwrap();
            let _ = client.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = [0u8; 1024];
            let _ = std::io::Read::read(&mut client, &mut buf);
        });

        let (stream, _) = listener.accept().unwrap();
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let result = handle_callback_connection(stream, "expected-state");

        handle.join().unwrap();

        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("state mismatch"),
            "Expected state mismatch: {msg}"
        );
    }

    #[test]
    fn callback_extracts_code_on_success() {
        use std::io::Write;
        use std::net::TcpStream;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = std::thread::spawn(move || {
            let mut client = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
            client
                .write_all(
                    b"GET /callback?code=my-auth-code&state=correct-state HTTP/1.1\r\nHost: localhost\r\n\r\n",
                )
                .unwrap();
            let _ = client.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = [0u8; 1024];
            let _ = std::io::Read::read(&mut client, &mut buf);
        });

        let (stream, _) = listener.accept().unwrap();
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let result = handle_callback_connection(stream, "correct-state");

        handle.join().unwrap();

        assert_eq!(result.unwrap(), "my-auth-code");
    }

    // === token expiry and auth flow edge cases ===

    #[test]
    fn oauth_tokens_expired_at_exact_boundary() {
        // Token expiring exactly now should be considered expired
        // because of the 60-second safety margin
        let tokens = OAuthTokens {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at: Some(Utc::now()),
            scopes: vec![],
        };
        assert!(
            tokens.is_expired(),
            "Token expiring at now should be expired (60s safety margin)"
        );
    }

    #[test]
    fn start_auth_flow_url_contains_required_params() {
        let config = OAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-secret".to_string(),
            redirect_port: 9999,
        };
        let (url, _verifier, _state) = start_auth_flow(&config).unwrap();
        assert!(
            url.contains("response_type=code"),
            "Missing response_type: {url}"
        );
        assert!(
            url.contains("client_id=test-client-id"),
            "Missing client_id: {url}"
        );
        assert!(
            url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A9999%2Fcallback"),
            "Missing or wrong redirect_uri: {url}"
        );
        assert!(
            url.contains("code_challenge="),
            "Missing code_challenge: {url}"
        );
        assert!(
            url.contains("code_challenge_method=S256"),
            "Missing code_challenge_method: {url}"
        );
        assert!(url.contains("state="), "Missing state: {url}");
        assert!(url.contains("scope="), "Missing scope: {url}");
    }

    #[test]
    fn oauth_config_with_custom_port() {
        let config = OAuthConfig {
            client_id: "cid".to_string(),
            client_secret: "csec".to_string(),
            redirect_port: 12345,
        };
        assert_eq!(config.redirect_port, 12345);
        let (url, _, _) = start_auth_flow(&config).unwrap();
        assert!(
            url.contains("12345"),
            "Custom port should appear in URL: {url}"
        );
    }
}
