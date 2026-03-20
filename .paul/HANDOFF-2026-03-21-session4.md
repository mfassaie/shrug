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
**Plan:** Not started

**Loop Position:**
```
PLAN ──▶ APPLY ──▶ UNIFY
  ○        ○        ○     [Ready for first PLAN]
```

---

## What Was Done (This Session)

- Plan 02-04: Spec analysis utilities + conformance test suite
  - URL builder, query string builder, pagination detection (Offset/Page/Cursor)
  - Path template validation, parameter helpers
  - Conformance tests with realistic Jira V3 (10 ops) + BitBucket V2 (5 ops) fixtures
  - 25 new tests
- Phase 2 transition: PROJECT.md evolved, ROADMAP.md updated, git commit d3ab516
- Plan 03-01: Product router — two-phase CLI parsing
  - operation_to_command_name (camelCase → kebab-case)
  - resolve_command: tag + operation matching with close-match suggestions
  - route_product: full routing pipeline
  - main.rs wired for all 5 products
  - 12 new tests
- Plan 03-02: Command tree display
  - Rich tag listing with descriptions and operation counts
  - Operation listing with method/summary/deprecated markers
  - Operation detail with parameter tables (required-first sorting)
  - Router error messages upgraded to use tree formatting
  - 6 new tests
- Phase 3 transition: git commit 8504c08
- All plans audited with enterprise audit workflow
- 137 tests passing, clippy clean, fmt clean
- User instruction: "don't stop, don't ask permissions, always audit"

---

## What's In Progress

- Nothing in progress — Phase 3 complete, Phase 4 not started

---

## What's Next

**Immediate:** Run `/paul:plan` for plan 04-01 (Profile management and config integration)

**Phase 4 scope (3 plans):**
- 04-01: Profile management and config integration
- 04-02: Keychain credential storage with encrypted file fallback
- 04-03: OAuth 2.0 flow, token refresh, and interactive setup wizard

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
| `.paul/ROADMAP.md` | 8-phase roadmap, Phases 1-3 complete |
| `.paul/PROJECT.md` | Full feature list, architecture, tech stack |
| `src/spec/` | Parsers (v3+v2), cache, registry, analysis |
| `src/cmd/router.rs` | Product routing, operation resolution |
| `src/cmd/tree.rs` | Command tree display formatting |
| `src/main.rs` | Entry point, wired to product router |
| `src/config.rs` | Config system with layered precedence |
| `Cargo.toml` | Dependencies: clap, tokio, reqwest, serde, chrono, etc. |

---

## Dev Environment Notes

- **Rust builds require MSYS2 GCC on PATH:** `export PATH="/c/msys64/mingw64/bin:$HOME/.cargo/bin:$PATH"`
- Rust 1.94.0, GNU target (x86_64-pc-windows-gnu)
- 1 flaky test: `config::tests::env_var_valid_overrides_apply` (env var race in parallel mode — passes with `--test-threads=1`)

---

## Resume Instructions

1. Read `.paul/STATE.md` for latest position
2. Phase 4 ready — run `/paul:resume` or `/paul:plan`
3. User wants autonomous execution: plan → audit → apply → unify → transition, no stopping

---

*Handoff created: 2026-03-21*
