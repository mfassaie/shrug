---
phase: 04-authentication-profiles
plan: 02
subsystem: auth
tags: [credentials, keychain, encryption, aes-gcm, argon2, keyring]

requires:
  - phase: 04-authentication-profiles
    provides: Profile CRUD (04-01), ProfileStore, AuthType enum
provides:
  - CredentialStore with keychain + encrypted file + env var backends
  - Auth CLI subcommands (set-token, status)
  - Credential lifecycle tied to profile delete
  - Credential resolution chain (env > keychain > encrypted file)
affects: [04-03 OAuth token storage, 05 HTTP executor auth header injection]

tech-stack:
  added: [keyring 3, aes-gcm 0.10, argon2 0.5, base64 0.22, rand 0.8, rpassword 7]
  patterns: [keychain-first with encrypted file fallback, CLI-layer prompts (no I/O in backends)]

key-files:
  created: [src/auth/credentials.rs]
  modified: [src/auth/mod.rs, src/cli.rs, src/main.rs, Cargo.toml]

key-decisions:
  - "Prompts in CLI layer only, backends receive parameters (audit fix)"
  - "has_credential returns Result<bool> not plain bool (audit fix)"
  - "No keychain probe in constructor — lazy fallback on first error (audit fix)"
  - "Enum dispatch for backends, not trait objects — simpler for 2 concrete types"
  - "CredentialStore is stateless per-call — tries keychain each time, falls back as needed"

patterns-established:
  - "Credential resolution: SHRUG_API_TOKEN env > keychain > encrypted file"
  - "AES-256-GCM + argon2 for encrypted file storage"
  - "Atomic write (temp-then-rename) for .enc files"
  - "resolve_profile_name() helper for CLI commands needing a profile arg"

duration: ~15min
started: 2026-03-21T09:30:00Z
completed: 2026-03-21T09:45:00Z
---

# Phase 4 Plan 02: Keychain Credential Storage Summary

**Credential storage with OS keychain primary, AES-256-GCM encrypted file fallback, and env var override for CI/CD.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15min |
| Tasks | 2 completed |
| Tests added | 13 |
| Total tests | 178 passing |
| Files modified | 5 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Store API token via keychain | Pass | store_keychain() tries Windows Credential Manager |
| AC-2: Retrieve credential for profile | Pass | resolve() returns ResolvedCredential with source |
| AC-3: Encrypted file fallback | Pass | AES-256-GCM + argon2, store/retrieve with password param |
| AC-4: Environment variable override | Pass | SHRUG_API_TOKEN, SHRUG_EMAIL, SHRUG_SITE override profile |
| AC-5: Delete credentials with profile | Pass | profile delete also calls cred_store.delete() (non-blocking) |
| AC-6: Credential status display | Pass | profile show includes "Token: set/not set" |
| AC-7: Keychain unavailable detection | Pass | store_keychain returns false, CLI falls back to encrypted |

## Accomplishments

- CredentialStore with keychain + encrypted file + env var resolution — 13 unit tests
- Auth CLI subcommands (set-token, status) with interactive prompts at CLI layer only
- Credential lifecycle integrated with profile delete (non-blocking cleanup)
- Profile show now displays token status without revealing the token value

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/auth/credentials.rs` | Created | CredentialStore, EncryptedBlob, encrypt/decrypt, resolve, 13 tests |
| `src/auth/mod.rs` | Modified | Added `pub mod credentials` |
| `src/cli.rs` | Modified | AuthCommands enum (SetToken, Status) |
| `src/main.rs` | Modified | handle_auth(), credential lifecycle, resolve_profile_name() |
| `Cargo.toml` | Modified | Added keyring, aes-gcm, argon2, base64, rand, rpassword |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| CLI-layer prompts only | Audit: backends must be testable without terminal I/O | Clean separation, enables unit testing with known passwords |
| has_credential returns Result | Audit: silent error suppression hides keychain problems | Users see real errors instead of false "not set" |
| No keychain probe | Audit: probe writes can leave garbage entries | Lazy fallback on first real operation |
| Stateless CredentialStore | CLI runs one command and exits — no need to cache backend state | Simpler code, each call tries keychain fresh |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 1 | Minor — cargo fmt formatting |
| Scope additions | 0 | None |
| Deferred | 0 | None |

**Total impact:** Trivial.

## Issues Encountered

None

## Next Phase Readiness

**Ready:**
- CredentialStore fully operational (keychain, encrypted file, env vars)
- Profile + credential lifecycle integrated
- Auth CLI subcommands working
- Ready for 04-03 (OAuth 2.0 flow) to extend credential types

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 04-authentication-profiles, Plan: 02*
*Completed: 2026-03-21*
