---
phase: 04-authentication-profiles
plan: 03
subsystem: auth
tags: [oauth2, pkce, token-refresh, setup-wizard, dialoguer, reqwest-blocking, sha2]

requires:
  - phase: 04-authentication-profiles
    provides: Profile CRUD (04-01), CredentialStore with keychain + encrypted file (04-02)
provides:
  - OAuth 2.0 (3LO) authorization code flow with PKCE
  - Automatic token refresh via ensure_fresh_tokens()
  - Interactive setup wizard (shrug auth setup)
  - OAuth login command (shrug auth login)
  - AuthScheme enum (Basic vs Bearer) for credential resolution
affects: [05 HTTP executor auth header injection, 08 E2E testing]

tech-stack:
  added: [sha2 0.10, url 2, open 5, dialoguer 0.11, reqwest blocking feature]
  patterns: [keychain-first with verify-read for OAuth storage, ensure_fresh_tokens as pre-resolve step, AuthScheme discriminated union]

key-files:
  created: [src/auth/oauth.rs]
  modified: [src/auth/credentials.rs, src/auth/mod.rs, src/cli.rs, src/main.rs, Cargo.toml]

key-decisions:
  - "Keychain-first with verify-read for OAuth tokens — prevents silent data loss (audit fix)"
  - "ensure_fresh_tokens() separated from resolve() — keeps resolve read-only (audit fix)"
  - "reqwest::blocking for token exchange/refresh — main is sync, no async runtime needed"
  - "derive_oauth_file_key() deterministic fallback — weaker than keychain but enables encrypted-at-rest"
  - "AuthScheme enum replaces separate email/api_token fields on ResolvedCredential"

patterns-established:
  - "OAuth storage: keychain primary (service 'shrug-oauth' / 'shrug-oauth-config'), encrypted .enc fallback"
  - "Callback server binds 127.0.0.1 explicitly, never 'localhost'"
  - "Error callback parsing: ?error= params surfaced as actionable AuthError messages"
  - "Interactive wizards use dialoguer for prompts, rpassword for secrets"

duration: ~20min
started: 2026-03-21T10:00:00Z
completed: 2026-03-21T10:20:00Z
---

# Phase 4 Plan 03: OAuth 2.0 Flow, Token Refresh, and Setup Wizard Summary

**OAuth 2.0 (3LO) with PKCE, automatic token refresh, and interactive setup wizard completing the Authentication & Profiles phase.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~20min |
| Tasks | 3 completed |
| Tests added | 25 |
| Total tests | 203 passing |
| Files modified | 6 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: OAuth Token Storage | Pass | Keychain-first + .oauth.enc fallback; verify-read prevents silent data loss |
| AC-2: OAuth 2.0 Authorization Code Flow with PKCE | Pass | Browser flow with 127.0.0.1 callback server, PKCE S256 |
| AC-2b: OAuth Callback Error Handling | Pass | ?error= params parsed, actionable AuthError returned |
| AC-3: Automatic Token Refresh | Pass | ensure_fresh_tokens() refreshes expired tokens before resolve() |
| AC-4: Interactive Setup Wizard | Pass | dialoguer prompts for profile + credentials, BasicAuth and OAuth2 paths |
| AC-5: OAuth Credential Resolution | Pass | AuthScheme::Bearer returned for OAuth2 profiles; resolve() is read-only |
| AC-6: Token Refresh as Pre-Step | Pass | ensure_fresh_tokens() called in CLI layer before resolve() |

## Accomplishments

- OAuth 2.0 3LO with PKCE — generate_pkce_pair(), start_auth_flow(), await_callback(), exchange_code(), refresh_tokens()
- AuthScheme enum (Basic/Bearer) replacing raw email/api_token fields on ResolvedCredential
- Keychain-first OAuth storage with verify-read and encrypted file fallback (no plaintext tokens on disk)
- ensure_fresh_tokens() as explicit pre-resolve step with 60s safety margin and reqwest::blocking
- Interactive setup wizard with dialoguer (profile name, site, email, auth type, credentials)
- `shrug auth login` and `shrug auth setup` commands wired and compiling
- 25 new tests: 13 in oauth.rs (PKCE, flow URL, token expiry, callback parsing), 12 in credentials.rs

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/auth/oauth.rs` | Created | OAuthTokens, OAuthConfig, PKCE, auth flow, callback server, token exchange/refresh, 13 tests |
| `src/auth/credentials.rs` | Modified | AuthScheme enum, OAuth store/retrieve (tokens + config), ensure_fresh_tokens, updated resolve/delete/has_credential, 12 new tests |
| `src/auth/mod.rs` | Modified | Added `pub mod oauth` |
| `src/cli.rs` | Modified | Added Login + Setup to AuthCommands |
| `src/main.rs` | Modified | handle_login(), handle_setup(), ensure_fresh_tokens pre-resolve wiring |
| `Cargo.toml` | Modified | Added sha2, url, open, dialoguer; reqwest blocking feature |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Keychain-first with verify-read | Audit: keychain store can silently fail; verify-read prevents data loss | Robust on Windows where keyring behavior varies |
| ensure_fresh_tokens separate from resolve | Audit: resolve should be read-only; refresh is HTTP side-effect | Clean architecture, observable via tracing |
| reqwest::blocking for OAuth HTTP | main() is sync; no async runtime needed for token exchange/refresh | Simple, no runtime conflicts |
| derive_oauth_file_key deterministic | When keychain unavailable, encrypted-at-rest is better than plaintext | Weaker than keychain but acceptable fallback |
| AuthScheme discriminated union | Prevents invalid state (Bearer with email, Basic without token) | Phase 5 can pattern-match for correct header construction |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 1 | Minor — keychain verify-read added for robustness |
| Scope additions | 0 | None |
| Deferred | 0 | None |

**Total impact:** Trivial — one robustness fix for platform-dependent keychain behavior.

### Auto-fixed Issues

**1. Keychain store verify-read for OAuth token/config storage**
- **Found during:** Task 1 (OAuth token storage tests)
- **Issue:** keyring::Entry::set_password() returns Ok on Windows but get_password() fails — tokens silently lost
- **Fix:** After keychain store, immediately verify with a read-back; fall through to encrypted file on failure
- **Verification:** All OAuth storage tests pass on Windows

## Issues Encountered

None

## Next Phase Readiness

**Ready:**
- Phase 4 (Authentication & Profiles) complete: profiles, credentials, OAuth 2.0, setup wizard
- AuthScheme enum ready for Phase 5 HTTP executor to construct Basic/Bearer headers
- ensure_fresh_tokens ready for pre-request token refresh in HTTP executor
- 203 tests passing, clippy clean, fmt clean

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 04-authentication-profiles, Plan: 03*
*Completed: 2026-03-21*
