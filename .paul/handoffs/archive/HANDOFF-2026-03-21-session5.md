# PAUL Handoff

**Date:** 2026-03-21
**Status:** Paused — context limit approaching

---

## READ THIS FIRST

You have no prior context. This document tells you everything.

**Project:** shrug — a dynamic CLI for Atlassian Cloud (Jira, Jira Software, Confluence, BitBucket, Service Management)
**Core value:** Users and AI agents can interact with Atlassian Cloud products from the command line without context-switching to a browser.

---

## Current State

**Version:** 0.1.0
**Phase:** 4 of 8 — Authentication & Profiles
**Plan:** 04-02 complete, 04-03 not started

**Loop Position:**
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ✓        ✓     [Loop complete — ready for next PLAN]
```

---

## What Was Done (This Session)

- Plan 04-01: Profile management and config integration
  - Profile struct, ProfileStore with TOML storage (one file per profile)
  - .default file for default profile tracking (single-source-of-truth)
  - CLI subcommands: create, list, show, delete, use
  - Profile resolution chain: --profile flag > SHRUG_PROFILE env > config > .default
  - Atomic writes (temp-then-rename), resilient list() on corrupted files
  - NotFound error (exit 4) for missing profiles
  - 27 new tests
- Plan 04-02: Keychain credential storage with encrypted file fallback
  - CredentialStore: keychain primary, AES-256-GCM encrypted file fallback, env var override
  - Auth CLI subcommands: set-token, status
  - Credential lifecycle tied to profile delete
  - Profile show displays token status (set/not set)
  - All interactive I/O in CLI layer, backends receive parameters (audit requirement)
  - has_credential returns Result<bool> (audit requirement)
  - No keychain probe — lazy fallback (audit requirement)
  - 13 new tests
- Both plans audited with enterprise audit workflow
- 178 tests passing, clippy clean, fmt clean
- User instruction: "don't stop, always audit"

---

## What's In Progress

- Nothing in progress — Plan 04-02 complete, Plan 04-03 not started

---

## What's Next

**Immediate:** Run `/paul:plan` for plan 04-03 (OAuth 2.0 flow, token refresh, and interactive setup wizard)

**Phase 4 scope (3 plans):**
- [x] 04-01: Profile management and config integration
- [x] 04-02: Keychain credential storage with encrypted file fallback
- [ ] 04-03: OAuth 2.0 flow, token refresh, and interactive setup wizard

**After Phase 4:**
- Phase 5: Generic HTTP Executor (4 plans)
- Phase 6: Output & Formatting (2 plans)
- Phase 7: Helper Commands & ADF (3 plans)
- Phase 8: Distribution & Polish (3 plans)

---

## Key Files

| File | Purpose |
|------|---------|
| `.paul/STATE.md` | Live project state |
| `.paul/ROADMAP.md` | 8-phase roadmap, Phases 1-3 complete, Phase 4 in progress |
| `.paul/PROJECT.md` | Full feature list, architecture, tech stack |
| `src/auth/profile.rs` | Profile struct, ProfileStore, 27 tests |
| `src/auth/credentials.rs` | CredentialStore, encrypted backend, env resolution, 13 tests |
| `src/cli.rs` | ProfileCommands, AuthCommands enums |
| `src/main.rs` | CLI wiring, handle_profile, handle_auth, resolve_profile |
| `src/cmd/router.rs` | Product routing, operation resolution |
| `src/cmd/tree.rs` | Command tree display formatting |
| `src/spec/` | Parsers (v3+v2), cache, registry, analysis |
| `Cargo.toml` | Dependencies including keyring, aes-gcm, argon2 |

---

## Dev Environment Notes

- **Rust builds require MSYS2 GCC on PATH:** `export PATH="/c/msys64/mingw64/bin:$HOME/.cargo/bin:$PATH"`
- Rust 1.94.0, GNU target (x86_64-pc-windows-gnu)
- 1 flaky test: `config::tests::env_var_valid_overrides_apply` (env var race in parallel mode — passes with `--test-threads=1`)

---

## Resume Instructions

1. Read `.paul/STATE.md` for latest position
2. Phase 4 in progress — 2/3 plans complete, run `/paul:resume`
3. User wants autonomous execution: plan → audit → apply → unify → transition, no stopping

---

*Handoff created: 2026-03-21*
