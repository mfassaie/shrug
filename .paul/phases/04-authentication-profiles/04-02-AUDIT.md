# Enterprise Plan Audit Report

**Plan:** .paul/phases/04-authentication-profiles/04-02-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable — now enterprise-ready after applying 1 must-have and 2 strongly-recommended fixes. The security architecture was sound from the start (keychain-first, AES-256-GCM, env override), but the plan embedded interactive I/O inside backend implementations, making the encrypted file backend untestable and coupling I/O to business logic. Fixed by moving all prompts to the CLI layer.

## 2. What Is Solid

- **Keychain-first with encrypted file fallback:** Correct layered security. OS keychain provides the strongest protection available on each platform; encrypted file is a reasonable fallback for headless environments.
- **AES-256-GCM + argon2:** Strong authenticated encryption. AES-GCM detects tampering. Argon2 is the recommended password hashing algorithm (PHC winner), resistant to GPU/ASIC attacks.
- **Environment variable override:** Correct for CI/CD. Env vars bypass all interactive prompts and backend complexity. The precedence chain (env > keychain > file) is well-ordered.
- **Credential lifecycle tied to profile delete:** Prevents orphaned secrets. Non-blocking warning on deletion failure is the right trade-off.
- **Clear scope boundaries:** No OAuth (04-03), no HTTP injection (Phase 5), API tokens only. Well-scoped.

## 3. Enterprise Gaps Identified

1. **Interactive prompts inside backend implementations.** `EncryptedFileBackend.store()` called `rpassword::prompt_password` directly. This means: (a) tests can't provide known passwords without terminal interaction, (b) non-interactive contexts (scripts, CI) can't use the backend, (c) I/O is coupled to business logic, violating separation of concerns.

2. **`has_credential` returns plain bool.** If the OS keychain is locked, broken, or has permission issues, `has_credential` silently returns `false`. The user sees "no credential" when the real issue is "keychain error." This hides real problems from diagnostics.

3. **Keychain probe in constructor.** Writing/deleting a probe entry during initialization can leave orphaned entries in the user's keychain if cleanup fails (crash, permission change between write and delete). Probing is an anti-pattern for keychain access.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Interactive prompts inside backends | CredentialBackend trait, EncryptedFileBackend, Task 2 | Moved all rpassword prompts to CLI layer (main.rs). Backends receive passwords as parameters. EncryptedFileBackend uses `store_with_password()` and `retrieve_with_password()` methods. Tests use known passwords directly. |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | `has_credential` returns bool | CredentialBackend trait | Changed to `Result<bool, ShrugError>` — surfaces real keychain errors instead of silently returning false |
| 2 | Keychain probe in constructor | KeychainBackend constructor | Removed probe. Constructor is `new() -> Self` (infallible). First real store/retrieve attempt triggers fallback on failure. Added lazy fallback flag. |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Token zeroization (secrecy/zeroize crate) | CLI tool exits immediately after command; token is in memory for milliseconds. Relevant for long-running server processes, not single-command CLI. |
| 2 | File permissions (0600) on .enc files | Files are in user's data_dir, content is already encrypted with AES-256-GCM. Double-protection is nice-to-have, not essential. |

## 5. Audit & Compliance Readiness

- **Audit evidence:** Credentials stored in OS keychain (platform audit trail) or encrypted files (inspectable artifacts with encrypted content).
- **Silent failure prevention:** `has_credential` now returns `Result`, surfacing keychain errors. Lazy fallback logs debug messages on keychain failure.
- **Post-incident reconstruction:** CredentialSource enum tracks where each credential came from (Keychain/EncryptedFile/Environment). Debug logging records resolution source.
- **Secrets handling:** Tokens never stored in plaintext on disk. Keychain uses OS-level protection. Encrypted files use AES-256-GCM with argon2-derived keys.

## 6. Final Release Bar

**What must be true:**
- All interactive I/O in CLI layer, backends receive parameters
- `has_credential` returns `Result<bool, ShrugError>`
- No keychain probe in constructor — lazy fallback on first error

**Remaining risks:** Encrypted file password has no complexity requirements (user could use "1234"). Acceptable for CLI tool where user chooses their own security posture.

**Sign-off:** I would sign my name to this system after the applied fixes are implemented correctly.

---

**Summary:** Applied 1 must-have + 2 strongly-recommended upgrades. Deferred 2 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
