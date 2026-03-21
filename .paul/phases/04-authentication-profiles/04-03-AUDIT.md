# Enterprise Plan Audit Report

**Plan:** .paul/phases/04-authentication-profiles/04-03-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

**Conditionally acceptable → Enterprise-ready after applying 2 must-have and 3 strongly-recommended fixes.**

The plan's architecture is sound — OAuth 2.0 3LO with PKCE, keychain-first storage, and a setup wizard is the right approach. However, the original plan had two compliance-blocking issues (plaintext token storage and client_secret inaccessible for auto-refresh) and three design gaps (loopback binding, error handling, sync/async confusion). All five have been remediated in the plan.

I would approve this plan for production after the applied fixes.

## 2. What Is Solid

- **PKCE implementation approach**: Generating verifier/challenge in-process with SHA-256, not relying on an external OAuth library, is appropriate for Atlassian's straightforward 3LO flow. Less dependency surface.
- **AuthScheme enum design**: Replacing separate email/api_token fields with `Basic { email, api_token } | Bearer { access_token }` is a clean discriminated union that makes it impossible to construct an invalid credential combination.
- **Boundary protection**: The plan correctly refuses to modify Profile struct fields, keeping OAuth config in separate storage. This preserves the 04-01 contract.
- **Scope limits**: Correctly deferring live API testing to Phase 8 and accessible-resources resolution to Phase 5. No scope creep.
- **Setup wizard as linear flow**: No branching menus, no state machine — appropriate for a CLI first-run experience.

## 3. Enterprise Gaps Identified

### 3.1 Plaintext OAuth Token Storage (CRITICAL)
The original plan stored OAuthTokens as `.oauth.json` — plaintext JSON containing access and refresh tokens. Refresh tokens are long-lived secrets that can mint unlimited access tokens. Plaintext storage violates PROJECT.md constraint: "Credentials must never be stored in plaintext."

### 3.2 OAuthConfig Encryption Prevents Auto-Refresh (CRITICAL)
Storing client_secret with `encrypt_token()` requires a user-supplied password for decryption. Since the CLI exits after each command, there's no session to cache the password. This makes automatic token refresh impossible — every API call would prompt for an encryption password. Defeats the purpose of AC-3.

### 3.3 Callback Server DNS Rebinding Risk
Using "localhost" for TCP binding can resolve to `::1` (IPv6) or be manipulated via DNS on shared/compromised systems. Must bind explicitly to `127.0.0.1`.

### 3.4 No OAuth Error Callback Handling
Atlassian returns `?error=access_denied&error_description=...` when the user denies consent. The plan only handled the happy path (`?code=...`). Without error handling, the callback server would hang until timeout on user denial.

### 3.5 Sync/Async Confusion in Task 2
Task 2 contained contradictory advice: "use tokio::net::TcpListener" then "main is NOT async, use std::net::TcpListener". The token exchange and refresh also need HTTP but the plan didn't clarify which reqwest variant. Resolved: all sync (`std::net::TcpListener` + `reqwest::blocking::Client`).

### 3.6 Token Refresh Inside resolve() Violates Separation of Concerns
resolve() was a pure read function in 04-02. Embedding HTTP calls (refresh) inside it creates side effects in a read path, makes testing harder, and introduces sync/async tension. Separated into `ensure_fresh_tokens()`.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | OAuth tokens stored as plaintext .oauth.json | Task 1 action (store_oauth_tokens), AC-1 | Changed to keychain-first + .oauth.enc encrypted fallback. Added "NEVER stored as plaintext" to AC-1. |
| 2 | OAuthConfig encryption prevents auto-refresh | Task 1 action (store_oauth_config, retrieve_oauth_config) | Changed to keychain-first for client_id/secret. Encrypted file only as fallback. Keychain enables passwordless auto-refresh. |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Callback server must bind 127.0.0.1 | Task 1 action (await_callback), AC-2, verification | Explicit `127.0.0.1:{port}` binding. Added verification check. |
| 2 | OAuth error callback handling missing | Task 1 action (await_callback), AC-2b added, Task 2 verify | Added error param parsing, AC-2b for denial flow, error test. |
| 3 | Token refresh separated from resolve() | Task 1 action (resolve + ensure_fresh_tokens), Task 2 action, AC-5 updated, AC-6 added, verification | resolve() is now read-only. ensure_fresh_tokens() is a separate CLI-layer pre-step. Added AC-6. |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Accessible-resources API call after OAuth to resolve cloud ID | Phase 5 (HTTP executor) will use the cloud ID. Not needed for token storage/refresh in Phase 4. |
| 2 | Constant-time state comparison in callback | Mentioned in plan fix. Low risk since state is single-use and short-lived. Implementation can use simple equality. |

## 5. Audit & Compliance Readiness

**Evidence production**: Token storage via keychain + encrypted file produces no plaintext artifacts on disk. OAuth flow uses PKCE which is auditable (verifier/challenge logged at debug level, not the values themselves).

**Silent failure prevention**: ensure_fresh_tokens() returns actionable errors. Callback error handling prevents silent hangs. has_credential() returns Result<bool> (from 04-02 audit).

**Post-incident reconstruction**: Token source is tracked via CredentialSource enum. Refresh events logged via tracing::info. Callback errors captured before CLI exit.

**Ownership**: All OAuth operations route through CredentialStore — single point of control for credential lifecycle. No direct keychain/file access outside this module.

## 6. Final Release Bar

**What must be true:**
- No plaintext token files exist on disk after any OAuth operation
- Auto-refresh works without user interaction when keychain is available
- Callback server binds to 127.0.0.1 only
- User denial at consent screen produces an actionable error, not a timeout

**Remaining risks if shipped as-is (after fixes):**
- Encrypted file fallback for OAuth requires password on every command (degraded UX, but secure)
- Port 8456 collision possible (acceptable — user can configure via OAuthConfig)
- No CSRF protection beyond state parameter (acceptable for local loopback)

**Sign-off:** With the applied fixes, this plan meets enterprise standards. I would sign my name to this system.

---

**Summary:** Applied 2 must-have + 3 strongly-recommended upgrades. Deferred 2 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
